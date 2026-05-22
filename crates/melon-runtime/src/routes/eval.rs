use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use melon_scenario::eval::EvalCase;
use serde::Serialize;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/tasks/{task_id}/eval", post(run_evals))
}

#[derive(Serialize)]
struct EvalResult {
    case_id: String,
    goal: String,
    passed: bool,
    details: Vec<String>,
}

#[derive(Serialize)]
struct EvalSummary {
    task_id: String,
    scenario_id: String,
    total: usize,
    passed: usize,
    failed: usize,
    results: Vec<EvalResult>,
}

async fn run_evals(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<EvalSummary>, StatusCode> {
    // Look up task to get scenario_id
    let scenario_id: Option<String> =
        sqlx::query_scalar("SELECT scenario_id FROM tasks WHERE id = ?")
            .bind(&task_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let scenario_id = scenario_id.ok_or(StatusCode::NOT_FOUND)?;

    // Resolve pack directory
    let pack_dir = resolve_pack_dir(&state.scenarios_dir, &scenario_id)?;

    // Load eval cases
    let evals_path = pack_dir.join("evals/cases.yaml");
    if !evals_path.exists() {
        return Ok(Json(EvalSummary {
            task_id,
            scenario_id,
            total: 0,
            passed: 0,
            failed: 0,
            results: Vec::new(),
        }));
    }

    let content =
        std::fs::read_to_string(&evals_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cases: Vec<EvalCase> =
        serde_yaml::from_str(&content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if cases.is_empty() {
        return Ok(Json(EvalSummary {
            task_id,
            scenario_id,
            total: 0,
            passed: 0,
            failed: 0,
            results: Vec::new(),
        }));
    }

    // Fetch all trace summaries for the explicit task
    let trace_summaries: Vec<String> = sqlx::query_scalar(
        "SELECT summary FROM trace_events WHERE task_id = ? ORDER BY timestamp ASC",
    )
    .bind(&task_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Join all summaries for pattern matching
    let combined: String = trace_summaries.join("\n").to_lowercase();

    // Evaluate each case
    let mut results = Vec::new();
    let mut passed_count = 0;

    for case in &cases {
        let mut details = Vec::new();
        let mut case_passed = true;

        // Check must_include
        for keyword in &case.expected.must_include {
            if combined.contains(&keyword.to_lowercase()) {
                details.push(format!("must_include '{}' found", keyword));
            } else {
                details.push(format!("must_include '{}' NOT found", keyword));
                case_passed = false;
            }
        }

        // Check must_not
        for keyword in &case.expected.must_not {
            if !combined.contains(&keyword.to_lowercase()) {
                details.push(format!("must_not '{}' absent (good)", keyword));
            } else {
                details.push(format!("must_not '{}' found (bad)", keyword));
                case_passed = false;
            }
        }

        if case_passed {
            passed_count += 1;
        }

        // Record to eval_runs
        let _ = sqlx::query(
            "INSERT INTO eval_runs (id, case_id, task_id, result, details) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&case.id)
        .bind(&task_id)
        .bind(if case_passed { "pass" } else { "fail" })
        .bind(serde_json::to_string(&details).unwrap_or_default())
        .execute(&state.db)
        .await;

        results.push(EvalResult {
            case_id: case.id.clone(),
            goal: case.goal.clone(),
            passed: case_passed,
            details,
        });
    }

    Ok(Json(EvalSummary {
        task_id,
        scenario_id,
        total: cases.len(),
        passed: passed_count,
        failed: cases.len() - passed_count,
        results,
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
