use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

#[derive(Serialize)]
struct TaskSummary {
    id: String,
    scenario_id: String,
    user_goal: String,
    status: String,
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    scenario_id: String,
    user_goal: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tasks", get(list_tasks))
        .route("/api/tasks", post(create_task))
}

async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<TaskSummary>>, StatusCode> {
    let tasks = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, scenario_id, user_goal, status FROM tasks ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list tasks: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        tasks
            .into_iter()
            .map(|(id, scenario_id, user_goal, status)| TaskSummary {
                id,
                scenario_id,
                user_goal,
                status,
            })
            .collect(),
    ))
}

async fn create_task(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<TaskSummary>, StatusCode> {
    if !scenario_exists(&state.scenarios_dir, &req.scenario_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let task_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO tasks (id, scenario_id, user_goal, status) VALUES (?, ?, ?, 'created')",
    )
    .bind(&task_id)
    .bind(&req.scenario_id)
    .bind(&req.user_goal)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create task: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Write initial trace event
    sqlx::query(
        "INSERT INTO trace_events (id, task_id, event_type, summary) VALUES (?, ?, 'system', 'Task created')"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&task_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to write trace event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(TaskSummary {
        id: task_id,
        scenario_id: req.scenario_id,
        user_goal: req.user_goal,
        status: "created".to_string(),
    }))
}

fn scenario_exists(scenarios_dir: &std::path::Path, scenario_id: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(scenarios_dir) else {
        return false;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("manifest.yaml");
        let Ok(content) = std::fs::read_to_string(&manifest_path) else {
            continue;
        };
        let Ok(manifest) = serde_yaml::from_str::<melon_scenario::manifest::Manifest>(&content)
        else {
            continue;
        };
        if manifest.id == scenario_id {
            return true;
        }
    }

    false
}
