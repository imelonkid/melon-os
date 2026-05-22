use anyhow::Result;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::path::Path;

/// Single entry from knowledge/sources.yaml.
#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub uri: String,
    #[serde(rename = "type")]
    pub source_type: String,
    pub description: Option<String>,
}

/// Top-level sources.yaml structure.
#[derive(Debug, Clone, Deserialize)]
pub struct SourcesFile {
    pub sources: Vec<KnowledgeSource>,
}

/// Load sources from a sources.yaml file.
pub fn load_sources(path: &Path) -> Result<SourcesFile> {
    let content = std::fs::read_to_string(path)?;
    let sources: SourcesFile = serde_yaml::from_str(&content)?;
    Ok(sources)
}

/// Hash file content for change detection.
fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Ingest all sources from a pack's knowledge directory into the database.
/// Returns the number of items ingested.
pub async fn ingest_pack(
    db: &SqlitePool,
    scenario_id: &str,
    knowledge_dir: &Path,
) -> Result<usize> {
    let sources_path = knowledge_dir.join("sources.yaml");
    if !sources_path.exists() {
        tracing::info!("No sources.yaml found in {:?}", knowledge_dir);
        return Ok(0);
    }

    let sources = load_sources(&sources_path)?;
    let mut count = 0;

    for source in &sources.sources {
        if source.source_type != "file" {
            tracing::warn!("Skipping non-file source type: {}", source.source_type);
            continue;
        }

        // Resolve file path: URIs are relative to pack root (parent of knowledge_dir)
        let pack_root = knowledge_dir.parent().unwrap_or(knowledge_dir);
        let file_path = pack_root.join(&source.uri);
        if !file_path.exists() {
            tracing::warn!(
                "Source file not found: {:?} (source_id={})",
                file_path,
                source.id
            );
            continue;
        }

        let content = std::fs::read_to_string(&file_path)?;
        let hash = content_hash(&content);

        // Check if already ingested (by scenario_id + source_id + hash)
        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM knowledge_items WHERE source_id = ? AND hash = ?")
                .bind(&source.id)
                .bind(&hash)
                .fetch_optional(db)
                .await?;

        if existing.is_some() {
            tracing::debug!("Source already ingested: {} (hash unchanged)", source.id);
            continue;
        }

        // Upsert source into knowledge_sources table
        sqlx::query(
            r#"
            INSERT INTO knowledge_sources (id, scenario_id, uri, source_type, description)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET uri = excluded.uri, source_type = excluded.source_type
            "#,
        )
        .bind(&source.id)
        .bind(scenario_id)
        .bind(&source.uri)
        .bind(&source.source_type)
        .bind(source.description.as_deref().unwrap_or(""))
        .execute(db)
        .await?;

        // Create knowledge item
        let item_id = uuid::Uuid::new_v4().to_string();
        let title = source
            .uri
            .rsplit('/')
            .next()
            .unwrap_or(&source.uri)
            .to_string();

        sqlx::query(
            "INSERT INTO knowledge_items (id, source_id, uri, title, content_type, hash) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&item_id)
        .bind(&source.id)
        .bind(&source.uri)
        .bind(&title)
        .bind("text/markdown")
        .bind(&hash)
        .execute(db)
        .await?;

        // Chunk content and store
        let chunks = chunk_text(&content, 500);
        for chunk in chunks.iter() {
            let chunk_id = uuid::Uuid::new_v4().to_string();
            let start_offset = chunk.1;
            let end_offset = chunk.1 + chunk.0.len();

            sqlx::query(
                "INSERT INTO knowledge_chunks (id, item_id, text, start_offset, end_offset) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&chunk_id)
            .bind(&item_id)
            .bind(&chunk.0)
            .bind(start_offset as i64)
            .bind(end_offset as i64)
            .execute(db)
            .await?;

            sqlx::query("INSERT INTO knowledge_fts (text, item_id, source_id) VALUES (?, ?, ?)")
                .bind(&chunk.0)
                .bind(&item_id)
                .bind(&source.id)
                .execute(db)
                .await?;
        }

        tracing::info!(
            "Ingested source: {} -> {} chunks from {}",
            source.id,
            chunks.len(),
            source.uri
        );
        count += 1;
    }

    Ok(count)
}

/// Search knowledge base by keyword.
/// Returns matching chunks with source_id, item_id, text, and chunk offset info.
pub struct SearchHit {
    pub source_id: String,
    pub item_id: String,
    pub uri: String,
    pub title: String,
    pub text: String,
    pub score: f64,
}

pub async fn search(db: &SqlitePool, query: &str) -> Result<Vec<SearchHit>> {
    let rows: Vec<(String, String, String, String, String, f64)> = sqlx::query_as(
        r#"
        SELECT fts.source_id, fts.item_id, ki.uri, ki.title, fts.text, fts.rank
        FROM knowledge_fts fts
        JOIN knowledge_items ki ON ki.id = fts.item_id
        WHERE knowledge_fts MATCH ?
        ORDER BY fts.rank
        LIMIT 20
        "#,
    )
    .bind(query)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(source_id, item_id, uri, title, text, rank)| SearchHit {
            source_id,
            item_id,
            uri,
            title,
            text,
            score: rank,
        })
        .collect())
}

pub async fn search_for_scenario(
    db: &SqlitePool,
    scenario_id: &str,
    query: &str,
) -> Result<Vec<SearchHit>> {
    let rows: Vec<(String, String, String, String, String, f64)> = sqlx::query_as(
        r#"
        SELECT fts.source_id, fts.item_id, ki.uri, ki.title, fts.text, fts.rank
        FROM knowledge_fts fts
        JOIN knowledge_items ki ON ki.id = fts.item_id
        JOIN knowledge_sources ks ON ks.id = ki.source_id
        WHERE knowledge_fts MATCH ? AND ks.scenario_id = ?
        ORDER BY fts.rank
        LIMIT 20
        "#,
    )
    .bind(query)
    .bind(scenario_id)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(source_id, item_id, uri, title, text, rank)| SearchHit {
            source_id,
            item_id,
            uri,
            title,
            text,
            score: rank,
        })
        .collect())
}

/// Simple keyword search fallback (when FTS is not available).
/// Returns chunks that contain any of the query keywords.
pub async fn search_keyword(db: &SqlitePool, query: &str) -> Result<Vec<SearchHit>> {
    // Try FTS first
    if let Ok(results) = search(db, query).await {
        if !results.is_empty() {
            return Ok(results);
        }
    }

    // Fallback: simple LIKE match on knowledge_chunks
    let pattern = format!("%{}%", query);
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT kc.text, ki.source_id, ki.id, ki.uri, ki.title
        FROM knowledge_chunks kc
        JOIN knowledge_items ki ON ki.id = kc.item_id
        WHERE kc.text LIKE ?
        LIMIT 20
        "#,
    )
    .bind(&pattern)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(text, source_id, item_id, uri, title)| SearchHit {
            source_id,
            item_id,
            uri,
            title,
            text,
            score: 1.0,
        })
        .collect())
}

pub async fn search_keyword_for_scenario(
    db: &SqlitePool,
    scenario_id: &str,
    query: &str,
) -> Result<Vec<SearchHit>> {
    if let Ok(results) = search_for_scenario(db, scenario_id, query).await {
        if !results.is_empty() {
            return Ok(results);
        }
    }

    let terms: Vec<String> = query
        .split_whitespace()
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect();
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT kc.text, ki.source_id, ki.id, ki.uri, ki.title
        FROM knowledge_chunks kc
        JOIN knowledge_items ki ON ki.id = kc.item_id
        JOIN knowledge_sources ks ON ks.id = ki.source_id
        WHERE ks.scenario_id = ?
        LIMIT 100
        "#,
    )
    .bind(scenario_id)
    .fetch_all(db)
    .await?;

    let mut hits = Vec::new();
    for (text, source_id, item_id, uri, title) in rows {
        let lower_text = text.to_lowercase();
        if terms.iter().any(|term| lower_text.contains(term)) {
            hits.push(SearchHit {
                source_id,
                item_id,
                uri,
                title,
                text,
                score: 1.0,
            });
        }
        if hits.len() >= 20 {
            break;
        }
    }

    Ok(hits)
}

/// Split text into chunks of approximately `max_chars` each.
/// Returns (chunk_text, start_offset) pairs.
fn chunk_text(text: &str, max_chars: usize) -> Vec<(String, usize)> {
    // Split by paragraph boundaries first, then by character limit
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_offset = 0;

    for para in paragraphs {
        if current.len() + para.len() > max_chars && !current.is_empty() {
            chunks.push((current.trim().to_string(), current_offset));
            current_offset += current.len();
            current.clear();
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para);
    }

    if !current.is_empty() {
        chunks.push((current.trim().to_string(), current_offset));
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[test]
    fn chunk_text_splits_by_paragraph() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = chunk_text(text, 500);
        assert_eq!(chunks.len(), 1); // all fit in one chunk at 500 chars
    }

    #[test]
    fn chunk_text_splits_long_paragraphs() {
        let text = "First.\n\nSecond paragraph that is longer.\n\nThird.\n\nFourth.";
        let chunks = chunk_text(text, 30);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn chunk_text_empty_paragraphs_ignored() {
        let text = "Hello.\n\n\n\nWorld.";
        let chunks = chunk_text(text, 500);
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn content_hash_is_deterministic() {
        let a = content_hash("hello");
        let b = content_hash("hello");
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn load_demo_ops_sources() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace = manifest_dir.parent().unwrap().parent().unwrap();
        let knowledge_dir = workspace.join("scenarios/demo-ops/knowledge");
        if !knowledge_dir.exists() {
            return;
        }

        let sources_file = knowledge_dir.join("sources.yaml");
        assert!(sources_file.exists(), "sources.yaml should exist");
        let sources = load_sources(&sources_file).expect("load sources");
        assert_eq!(sources.sources.len(), 1);
        assert_eq!(sources.sources[0].id, "inspection_runbook");
    }

    #[tokio::test]
    async fn ingest_demo_ops_pack() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace = manifest_dir.parent().unwrap().parent().unwrap();
        let knowledge_dir = workspace.join("scenarios/demo-ops/knowledge");
        if !knowledge_dir.exists() {
            return;
        }

        // Create temp DB
        let db_path =
            std::env::temp_dir().join(format!("melon-kb-test-{}.db", uuid::Uuid::new_v4()));
        std::fs::File::create(&db_path).unwrap();
        let db = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();

        // Create tables
        sqlx::query(
            "CREATE TABLE knowledge_sources (id TEXT PRIMARY KEY, scenario_id TEXT NOT NULL, uri TEXT NOT NULL, source_type TEXT NOT NULL, description TEXT)",
        ).execute(&db).await.unwrap();
        sqlx::query(
            "CREATE TABLE knowledge_items (id TEXT PRIMARY KEY, source_id TEXT NOT NULL, uri TEXT NOT NULL, title TEXT, content_type TEXT, hash TEXT NOT NULL)",
        ).execute(&db).await.unwrap();
        sqlx::query(
            "CREATE TABLE knowledge_chunks (id TEXT PRIMARY KEY, item_id TEXT NOT NULL, text TEXT NOT NULL, start_offset INTEGER, end_offset INTEGER)",
        ).execute(&db).await.unwrap();
        sqlx::query("CREATE VIRTUAL TABLE knowledge_fts USING fts5(text, item_id, source_id)")
            .execute(&db)
            .await
            .unwrap();

        let count = ingest_pack(&db, "demo.ops", &knowledge_dir)
            .await
            .expect("ingest pack");
        assert_eq!(count, 1, "Should ingest 1 source");

        // Verify sources
        let sources: Vec<String> = sqlx::query_scalar("SELECT id FROM knowledge_sources")
            .fetch_all(&db)
            .await
            .unwrap();
        assert!(sources.contains(&"inspection_runbook".to_string()));

        // Verify items
        let items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_items")
            .fetch_one(&db)
            .await
            .unwrap();
        assert!(items > 0, "Should have knowledge items");

        // Verify chunks
        let chunks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_chunks")
            .fetch_one(&db)
            .await
            .unwrap();
        assert!(chunks > 0, "Should have knowledge chunks");

        let fts_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_fts")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(fts_rows, chunks, "FTS should retain every ingested chunk");

        // Verify search
        let hits = search_keyword_for_scenario(&db, "demo.ops", "storage")
            .await
            .unwrap();
        assert!(!hits.is_empty(), "Should find storage-related chunks");

        sqlx::query(
            "INSERT INTO knowledge_sources (id, scenario_id, uri, source_type, description) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("other_source")
        .bind("other.pack")
        .bind("knowledge/fixtures/other.md")
        .bind("file")
        .bind("Other source")
        .execute(&db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO knowledge_items (id, source_id, uri, title, content_type, hash) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind("other_item")
        .bind("other_source")
        .bind("knowledge/fixtures/other.md")
        .bind("other.md")
        .bind("text/markdown")
        .bind("other_hash")
        .execute(&db)
        .await
        .unwrap();
        sqlx::query("INSERT INTO knowledge_chunks (id, item_id, text, start_offset, end_offset) VALUES (?, ?, ?, ?, ?)")
            .bind("other_chunk")
            .bind("other_item")
            .bind("storage content from another scenario")
            .bind(0_i64)
            .bind(37_i64)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO knowledge_fts (text, item_id, source_id) VALUES (?, ?, ?)")
            .bind("storage content from another scenario")
            .bind("other_item")
            .bind("other_source")
            .execute(&db)
            .await
            .unwrap();

        let isolated_hits = search_keyword_for_scenario(&db, "demo.ops", "storage")
            .await
            .unwrap();
        assert!(
            isolated_hits
                .iter()
                .all(|hit| hit.source_id == "inspection_runbook"),
            "Scenario search should not leak other pack results: {:?}",
            isolated_hits
                .iter()
                .map(|hit| &hit.source_id)
                .collect::<Vec<_>>()
        );
    }
}
