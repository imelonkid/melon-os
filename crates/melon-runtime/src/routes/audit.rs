use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/tasks/{task_id}/audit", get(list_audit_logs))
}

#[derive(Serialize, sqlx::FromRow)]
struct AuditLogEntry {
    id: String,
    task_id: String,
    scenario_id: String,
    action: String,
    input_summary: String,
    output_summary: String,
    approval_status: String,
    timestamp: String,
}

async fn list_audit_logs(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<AuditLogEntry>>, StatusCode> {
    let logs: Vec<AuditLogEntry> = sqlx::query_as(
        "SELECT id, task_id, scenario_id, action, input_summary, output_summary, approval_status, timestamp FROM audit_logs WHERE task_id = ? ORDER BY timestamp ASC",
    )
    .bind(&task_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(logs))
}
