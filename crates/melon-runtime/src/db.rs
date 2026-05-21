use anyhow::Result;
use sqlx::SqlitePool;
use std::path::Path;

pub async fn init_db(path: &str) -> Result<SqlitePool> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Touch the database file if it doesn't exist
    if !Path::new(path).exists() {
        std::fs::File::create(path)?;
    }

    let db_url = format!("sqlite:{}", path);
    let pool = SqlitePool::connect(&db_url).await?;

    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scenario_packs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            description TEXT,
            path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            scenario_id TEXT NOT NULL,
            user_goal TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'created',
            plan TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS trace_events (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            summary TEXT,
            input_ref TEXT,
            output_ref TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES tasks(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS approval_requests (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            action TEXT NOT NULL,
            risk_level TEXT NOT NULL,
            scope TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            resolved_at DATETIME,
            FOREIGN KEY (task_id) REFERENCES tasks(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tools (
            id TEXT PRIMARY KEY,
            tool_type TEXT NOT NULL,
            config TEXT,
            status TEXT NOT NULL DEFAULT 'stopped',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS permissions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            action TEXT NOT NULL,
            default_action TEXT NOT NULL,
            scopes TEXT,
            scenario_id TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS knowledge_sources (
            id TEXT PRIMARY KEY,
            scenario_id TEXT NOT NULL,
            uri TEXT NOT NULL,
            source_type TEXT NOT NULL,
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS knowledge_items (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL,
            uri TEXT NOT NULL,
            title TEXT,
            content_type TEXT,
            hash TEXT NOT NULL,
            metadata TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (source_id) REFERENCES knowledge_sources(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS knowledge_chunks (
            id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL,
            text TEXT NOT NULL,
            start_offset INTEGER,
            end_offset INTEGER,
            citations TEXT,
            FOREIGN KEY (item_id) REFERENCES knowledge_items(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS eval_cases (
            id TEXT PRIMARY KEY,
            scenario_id TEXT NOT NULL,
            goal TEXT NOT NULL,
            expected TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS eval_runs (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            result TEXT NOT NULL,
            details TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (case_id) REFERENCES eval_cases(id),
            FOREIGN KEY (task_id) REFERENCES tasks(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            scenario_id TEXT,
            tool_id TEXT,
            action TEXT NOT NULL,
            input_summary TEXT,
            output_summary TEXT,
            approval_status TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            rollback_hint TEXT,
            FOREIGN KEY (task_id) REFERENCES tasks(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ui_sessions (
            id TEXT PRIMARY KEY,
            layout TEXT,
            state TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    tracing::info!("Database initialized at {}", path);

    Ok(pool)
}
