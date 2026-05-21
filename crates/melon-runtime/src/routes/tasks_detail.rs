use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use melon_agent::executor::resolve_approval;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tasks/{task_id}", get(get_task))
        .route("/api/tasks/{task_id}/traces", get(list_traces))
        .route("/api/tasks/{task_id}/approvals", get(list_approvals))
        .route(
            "/api/tasks/{task_id}/approvals/{approval_id}/action",
            post(resolve_approval_endpoint),
        )
}

// --- GET /api/tasks/{task_id} ---

#[derive(Serialize)]
struct TaskDetail {
    id: String,
    scenario_id: String,
    user_goal: String,
    status: String,
}

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskDetail>, StatusCode> {
    let row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, scenario_id, user_goal, status FROM tasks WHERE id = ?",
    )
    .bind(&task_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(TaskDetail {
        id: row.0,
        scenario_id: row.1,
        user_goal: row.2,
        status: row.3,
    }))
}

// --- GET /api/tasks/{task_id}/traces ---

#[derive(Serialize)]
struct TraceEvent {
    id: String,
    event_type: String,
    summary: String,
    input_ref: Option<String>,
    output_ref: Option<String>,
    timestamp: String,
}

async fn list_traces(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<TraceEvent>>, StatusCode> {
    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, String)>(
        "SELECT id, event_type, summary, input_ref, output_ref, timestamp FROM trace_events WHERE task_id = ? ORDER BY timestamp ASC",
    )
    .bind(&task_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch traces for task {}: {}", task_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        rows.into_iter()
            .map(|r| TraceEvent {
                id: r.0,
                event_type: r.1,
                summary: r.2,
                input_ref: r.3,
                output_ref: r.4,
                timestamp: r.5,
            })
            .collect(),
    ))
}

// --- GET /api/tasks/{task_id}/approvals ---

#[derive(Serialize)]
struct ApprovalItem {
    id: String,
    action: String,
    risk_level: String,
    scope: String,
    status: String,
}

async fn list_approvals(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<ApprovalItem>>, StatusCode> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT id, action, risk_level, scope, status FROM approval_requests WHERE task_id = ?",
    )
    .bind(&task_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch approvals for task {}: {}", task_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        rows.into_iter()
            .map(|r| ApprovalItem {
                id: r.0,
                action: r.1,
                risk_level: r.2,
                scope: r.3,
                status: r.4,
            })
            .collect(),
    ))
}

// --- POST /api/tasks/{task_id}/approvals/{approval_id}/action ---

#[derive(Deserialize)]
struct ApprovalAction {
    action: String, // "approve" or "reject"
}

#[derive(Serialize)]
struct ApprovalActionResponse {
    approval_id: String,
    result: String,
}

async fn resolve_approval_endpoint(
    State(state): State<AppState>,
    Path((task_id, approval_id)): Path<(String, String)>,
    Json(req): Json<ApprovalAction>,
) -> Result<Json<ApprovalActionResponse>, StatusCode> {
    // Verify the approval belongs to this task
    let exists: Option<String> = sqlx::query_scalar(
        "SELECT id FROM approval_requests WHERE id = ? AND task_id = ? AND status = 'pending'",
    )
    .bind(&approval_id)
    .bind(&task_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let approved = match req.action.as_str() {
        "approve" => true,
        "reject" => false,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // Update database
    let new_status = if approved { "approved" } else { "rejected" };
    sqlx::query(
        "UPDATE approval_requests SET status = ?, resolved_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(new_status)
    .bind(&approval_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Try to notify in-memory executor via channel
    resolve_approval(&state.approval_channels, &approval_id, approved).await;

    Ok(Json(ApprovalActionResponse {
        approval_id,
        result: new_status.to_string(),
    }))
}
