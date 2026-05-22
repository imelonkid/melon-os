use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use melon_tools::mock_adapter::MockToolAdapter;
use melon_tools::registry::ToolRegistry;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/packs/{pack_id}/run", post(run_pack))
}

#[derive(Deserialize)]
struct RunRequest {
    user_goal: String,
}

#[derive(Serialize)]
struct RunResponse {
    task_id: String,
    scenario_id: String,
    status: String,
    user_goal: String,
}

async fn run_pack(
    State(state): State<AppState>,
    Path(pack_id): Path<String>,
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, StatusCode> {
    // Resolve pack directory
    let pack_dir = resolve_pack_dir(&state.scenarios_dir, &pack_id)?;

    // Create task in database
    let task_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO tasks (id, scenario_id, user_goal, status) VALUES (?, ?, ?, 'created')",
    )
    .bind(&task_id)
    .bind(&pack_id)
    .bind(&req.user_goal)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Write initial trace
    sqlx::query(
        "INSERT INTO trace_events (id, task_id, event_type, summary) VALUES (?, ?, 'system', 'Task created by Run')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&task_id)
    .execute(&state.db)
    .await
    .ok();

    // Spawn workflow executor as background task
    let mut registry = ToolRegistry::new();
    registry.register_adapter(Arc::new(MockToolAdapter::new()));
    let registry = Arc::new(registry);

    let ctx = melon_agent::executor::ExecutorContext {
        db: state.db.clone(),
        pack_dir,
        task_id: task_id.clone(),
        registry,
    };

    tokio::spawn(async move {
        if let Err(e) = melon_agent::executor::execute_workflow(ctx).await {
            tracing::error!("Workflow execution failed: {}", e);
        }
    });

    Ok(Json(RunResponse {
        task_id,
        scenario_id: pack_id,
        status: "created".to_string(),
        user_goal: req.user_goal,
    }))
}

fn resolve_pack_dir(
    scenarios_dir: &std::path::PathBuf,
    pack_id: &str,
) -> Result<std::path::PathBuf, StatusCode> {
    let entries = std::fs::read_dir(scenarios_dir).map_err(|_| StatusCode::NOT_FOUND)?;

    for entry in entries {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if entry.path().is_dir() {
            let manifest_path = entry.path().join("manifest.yaml");
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(manifest) =
                    serde_yaml::from_str::<melon_scenario::manifest::Manifest>(&content)
                {
                    if manifest.id == pack_id {
                        return Ok(entry.path());
                    }
                }
            }
        }
    }

    Err(StatusCode::NOT_FOUND)
}
