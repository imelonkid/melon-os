use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/packs/{pack_id}/knowledge/search",
            get(search_knowledge),
        )
        .route("/api/packs/{pack_id}/knowledge/sources", get(list_sources))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Serialize)]
struct SearchHit {
    source_id: String,
    item_id: String,
    uri: String,
    title: String,
    text: String,
    score: f64,
}

async fn search_knowledge(
    State(state): State<AppState>,
    Path(pack_id): Path<String>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchHit>>, StatusCode> {
    let hits = melon_kb::search_keyword_for_scenario(&state.db, &pack_id, &params.q)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|h| SearchHit {
            source_id: h.source_id,
            item_id: h.item_id,
            uri: h.uri,
            title: h.title,
            text: h.text,
            score: h.score,
        })
        .collect();

    Ok(Json(hits))
}

#[derive(Serialize, sqlx::FromRow)]
struct SourceEntry {
    id: String,
    uri: String,
    source_type: String,
    description: Option<String>,
}

async fn list_sources(
    State(state): State<AppState>,
    Path(pack_id): Path<String>,
) -> Result<Json<Vec<SourceEntry>>, StatusCode> {
    let sources: Vec<SourceEntry> = sqlx::query_as(
        "SELECT id, uri, source_type, description FROM knowledge_sources WHERE scenario_id = ?",
    )
    .bind(&pack_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(sources))
}
