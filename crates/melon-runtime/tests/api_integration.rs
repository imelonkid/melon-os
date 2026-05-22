use axum::{body::Body, http::Request, Router};
use http::StatusCode;
use melon_runtime::{db, AppState};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

fn test_scenarios_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("melon-test-scenarios-{}", uuid::Uuid::new_v4()))
}

fn write_manifest(dir: &std::path::Path, pack_id: &str) {
    let pack_dir = dir.join(pack_id);
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(
        pack_dir.join("manifest.yaml"),
        format!("id: {pack_id}\nname: {pack_id}\nversion: 0.1.0\n"),
    )
    .unwrap();
    fs::write(pack_dir.join("role.md"), "# Role\n").unwrap();
}

async fn test_app() -> (Router, AppState) {
    let tmp_dir = std::env::temp_dir();
    fs::create_dir_all(&tmp_dir).unwrap();
    let db_path = format!(
        "{}/melon-test-{}.db",
        tmp_dir.display(),
        uuid::Uuid::new_v4()
    );
    let pool = db::init_db(&db_path).await.expect("init db");

    let scenarios_dir = test_scenarios_dir();
    fs::create_dir_all(&scenarios_dir).unwrap();

    let state = AppState {
        db: pool,
        scenarios_dir,
        approval_channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .merge(melon_runtime::routes::packs::router())
        .merge(melon_runtime::routes::tasks::router())
        .merge(melon_runtime::routes::tasks_detail::router())
        .merge(melon_runtime::routes::files::router())
        .merge(melon_runtime::routes::validate::router())
        .merge(melon_runtime::routes::run::router())
        .merge(melon_runtime::routes::audit::router())
        .merge(melon_runtime::routes::eval::router())
        .with_state(state.clone());

    (app, state)
}

#[tokio::test]
async fn test_packs_includes_invalid_manifest_with_id() {
    let (app, state) = test_app().await;
    let pack_dir = state.scenarios_dir.join("bad.pack");
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(
        pack_dir.join("manifest.yaml"),
        "id: bad.pack\nversion: 0.1.0\n",
    )
    .unwrap();
    fs::write(pack_dir.join("role.md"), "# Role\n").unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/packs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let packs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0]["id"], "bad.pack");
    assert_eq!(packs[0]["valid"], false);
    assert!(packs[0]["validation_errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|error| error.as_str().unwrap_or_default().contains("name")));
}

#[tokio::test]
async fn test_validate_endpoint_reports_invalid_manifest_with_id() {
    let (app, state) = test_app().await;
    let pack_dir = state.scenarios_dir.join("bad.pack");
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(
        pack_dir.join("manifest.yaml"),
        "id: bad.pack\nversion: 0.1.0\n",
    )
    .unwrap();
    fs::write(pack_dir.join("role.md"), "# Role\n").unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/bad.pack/validate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["valid"], false);
    assert!(response["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|error| error.as_str().unwrap_or_default().contains("name")));
}

#[tokio::test]
async fn test_validate_endpoint_can_address_manifest_missing_id_by_directory_name() {
    let (app, state) = test_app().await;
    let pack_dir = state.scenarios_dir.join("bad-pack-dir");
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(
        pack_dir.join("manifest.yaml"),
        "name: Bad Pack\nversion: 0.1.0\n",
    )
    .unwrap();
    fs::write(pack_dir.join("role.md"), "# Role\n").unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/bad-pack-dir/validate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["valid"], false);
    assert!(response["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|error| error.as_str().unwrap_or_default().contains("id")));
}

#[tokio::test]
async fn test_packs_empty_when_no_scenarios() {
    let (app, _) = test_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/packs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), b"[]");
}

#[tokio::test]
async fn test_packs_returns_discovered_packs() {
    let (app, state) = test_app().await;
    write_manifest(&state.scenarios_dir, "test.pack");

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/packs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let packs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0]["id"], "test.pack");
    assert_eq!(packs[0]["valid"], true);
}

#[tokio::test]
async fn test_create_task_persists() {
    let (app, state) = test_app().await;
    let app2 = app.clone();
    write_manifest(&state.scenarios_dir, "test.pack");

    let body = Body::from(
        serde_json::to_string(&json!({
            "scenario_id": "test.pack",
            "user_goal": "Test goal"
        }))
        .unwrap(),
    );

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/tasks")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(task["status"], "created");
    assert_eq!(task["user_goal"], "Test goal");
    assert!(!task["id"].as_str().unwrap().is_empty());

    // Verify task is queryable
    let res = app2
        .oneshot(
            Request::builder()
                .uri("/api/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(tasks.len(), 1);
}

#[tokio::test]
async fn test_create_task_rejects_missing_scenario() {
    let (app, _) = test_app().await;

    let body = Body::from(
        serde_json::to_string(&json!({
            "scenario_id": "nonexistent.pack",
            "user_goal": "Test goal"
        }))
        .unwrap(),
    );

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/tasks")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_pack_files_list() {
    let (app, state) = test_app().await;
    write_manifest(&state.scenarios_dir, "test.pack");
    let workflows_dir = state.scenarios_dir.join("test.pack/workflows");
    fs::create_dir_all(&workflows_dir).unwrap();
    fs::write(
        workflows_dir.join("default.yaml"),
        "name: test\nsteps: []\n",
    )
    .unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/packs/test.pack/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_save_file_cannot_escape_pack_directory() {
    let (app, state) = test_app().await;
    write_manifest(&state.scenarios_dir, "test.pack");

    // Try to write to a path with parent dir traversal
    let body = Body::from(
        serde_json::to_string(&json!({
            "content": "malicious content"
        }))
        .unwrap(),
    );

    let res = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/packs/test.pack/files/..%2F..%2Fevil.yaml")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_save_file_cannot_use_absolute_path() {
    let (app, state) = test_app().await;
    write_manifest(&state.scenarios_dir, "test.pack");

    let body = Body::from(
        serde_json::to_string(&json!({
            "content": "malicious content"
        }))
        .unwrap(),
    );

    let res = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/packs/test.pack/files/%2Fetc%2Fpasswd")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be forbidden for absolute path
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

// --- Run/Debug loop tests ---

fn write_full_test_pack(dir: &std::path::Path, pack_id: &str, approval_in_workflow: bool) {
    let pack_dir = dir.join(pack_id);
    fs::create_dir_all(&pack_dir).unwrap();
    fs::create_dir_all(pack_dir.join("workflows")).unwrap();
    fs::create_dir_all(pack_dir.join("tools")).unwrap();
    fs::create_dir_all(pack_dir.join("permissions")).unwrap();
    fs::create_dir_all(pack_dir.join("evals")).unwrap();

    // manifest.yaml
    fs::write(
        pack_dir.join("manifest.yaml"),
        format!("id: {pack_id}\nname: {pack_id}\nversion: 0.1.0\nentry: workflows/default.yaml\n"),
    )
    .unwrap();

    // role.md
    fs::write(pack_dir.join("role.md"), "# Test Role\n").unwrap();

    // workflows/default.yaml
    let workflow = if approval_in_workflow {
        "name: test_approval
steps:
  - id: check_service
    type: tool
    action: mock_check_service
    policy_action: read_status
  - id: cleanup
    type: tool
    action: mock_cleanup_temp
    policy_action: write_cleanup
    approval: cleanup_files
  - id: report
    type: ui.document
"
    } else {
        "name: test_no_approval
steps:
  - id: check_service
    type: tool
    action: mock_check_service
    policy_action: read_status
  - id: report
    type: ui.document
"
    };
    fs::write(pack_dir.join("workflows/default.yaml"), workflow).unwrap();

    // tools/tools.yaml
    fs::write(
        pack_dir.join("tools/tools.yaml"),
        "id: mock_check_service\ntype: mock\n",
    )
    .unwrap();

    // permissions/policy.yaml
    fs::write(
        pack_dir.join("permissions/policy.yaml"),
        "policies:\n  read_status:\n    default: allow\n  write_cleanup:\n    default: ask\naudit:\n  enabled: true\n",
    )
    .unwrap();

    // evals/cases.yaml
    fs::write(
        pack_dir.join("evals/cases.yaml"),
        "- id: test-001\n  goal: Run workflow\n  expected:\n    must_include:\n      - running\n      - completed\n    must_not:\n      - error\n",
    )
    .unwrap();
}

async fn poll_task_status(app: &Router, task_id: &str, expected_statuses: &[&str]) -> String {
    for _ in 0..50 {
        // up to 5 seconds (100ms * 50)
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/tasks/{}", task_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        if res.status() == StatusCode::OK {
            let body = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap();
            let task: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let status = task["status"].as_str().unwrap_or("").to_string();
            if expected_statuses.contains(&status.as_str()) {
                return status;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    "timeout".to_string()
}

#[tokio::test]
async fn test_run_creates_task_with_trace() {
    let (app, state) = test_app().await;
    write_full_test_pack(&state.scenarios_dir, "test.run-pack", false);

    let body = Body::from(
        serde_json::to_string(&json!({
            "user_goal": "Execute test workflow"
        }))
        .unwrap(),
    );

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/test.run-pack/run")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = response["task_id"].as_str().unwrap().to_string();
    assert_eq!(response["scenario_id"], "test.run-pack");

    // Wait for task to complete (no approval needed)
    let status = poll_task_status(&app, &task_id, &["completed"]).await;
    assert_eq!(status, "completed");

    // Verify trace events exist
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/tasks/{}/traces", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let traces: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!traces.is_empty(), "Should have trace events");
    // Should contain workflow start and completion traces
    let summaries: Vec<_> = traces
        .iter()
        .map(|t| t["summary"].as_str().unwrap_or(""))
        .collect();
    assert!(
        summaries.iter().any(|s| s.contains("Starting workflow")),
        "Should have workflow start trace"
    );
    assert!(
        summaries.iter().any(|s| s.contains("Workflow completed")),
        "Should have workflow completed trace: {:?}",
        summaries
    );
}

#[tokio::test]
async fn test_run_approval_approve_completes_task() {
    let (app, state) = test_app().await;
    write_full_test_pack(&state.scenarios_dir, "test.approval-pack", true);

    // Run the pack
    let body = Body::from(
        serde_json::to_string(&json!({
            "user_goal": "Execute with approval"
        }))
        .unwrap(),
    );

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/test.approval-pack/run")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = response["task_id"].as_str().unwrap().to_string();

    // Wait for awaiting_approval
    let status = poll_task_status(&app, &task_id, &["awaiting_approval"]).await;
    assert_eq!(status, "awaiting_approval");

    // Get the pending approval
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/tasks/{}/approvals", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let approvals: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!approvals.is_empty(), "Should have pending approval");
    let approval_id = approvals[0]["id"].as_str().unwrap().to_string();
    assert_eq!(approvals[0]["action"], "cleanup_files");
    assert_eq!(approvals[0]["risk_level"], "medium");

    // Approve
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/tasks/{}/approvals/{}/action",
                    task_id, approval_id
                ))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({ "action": "approve" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Wait for completion
    let status = poll_task_status(&app, &task_id, &["completed"]).await;
    assert_eq!(status, "completed");

    // Verify audit logs exist
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/tasks/{}/traces", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let traces: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    // Should have more traces after approval (cleanup, report, completion)
    assert!(
        traces.len() >= 5,
        "Should have multiple traces after approval: {:?}",
        traces
    );
}

#[tokio::test]
async fn test_run_approval_reject_cancels_task() {
    let (app, state) = test_app().await;
    write_full_test_pack(&state.scenarios_dir, "test.reject-pack", true);

    // Run the pack
    let body = Body::from(
        serde_json::to_string(&json!({
            "user_goal": "Execute and reject"
        }))
        .unwrap(),
    );

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/test.reject-pack/run")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = response["task_id"].as_str().unwrap().to_string();

    // Wait for awaiting_approval
    let status = poll_task_status(&app, &task_id, &["awaiting_approval"]).await;
    assert_eq!(status, "awaiting_approval");

    // Get the pending approval
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/tasks/{}/approvals", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let approvals: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let approval_id = approvals[0]["id"].as_str().unwrap().to_string();

    // Reject
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/tasks/{}/approvals/{}/action",
                    task_id, approval_id
                ))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({ "action": "reject" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Wait for cancellation
    let status = poll_task_status(&app, &task_id, &["cancelled"]).await;
    assert_eq!(status, "cancelled");
}

#[tokio::test]
async fn test_eval_runner_passes_on_valid_workflow() {
    let (app, state) = test_app().await;
    write_full_test_pack(&state.scenarios_dir, "test.eval-pack", false);

    // Run the workflow first
    let body = Body::from(
        serde_json::to_string(&json!({
            "user_goal": "Execute for eval"
        }))
        .unwrap(),
    );

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/test.eval-pack/run")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = response["task_id"].as_str().unwrap().to_string();

    // Wait for completion
    let status = poll_task_status(&app, &task_id, &["completed"]).await;
    assert_eq!(status, "completed");

    // Run eval
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/tasks/{}/eval", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(summary["total"], 1);
    assert_eq!(summary["passed"], 1);
    assert_eq!(summary["failed"], 0);
    assert_eq!(summary["results"][0]["passed"], true);
}

#[tokio::test]
async fn test_eval_runner_fails_on_rejected_workflow() {
    let (app, state) = test_app().await;
    write_full_test_pack(&state.scenarios_dir, "test.eval-reject-pack", true);

    // Run the workflow
    let body = Body::from(
        serde_json::to_string(&json!({
            "user_goal": "Execute and reject for eval"
        }))
        .unwrap(),
    );

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/test.eval-reject-pack/run")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = response["task_id"].as_str().unwrap().to_string();

    // Wait for awaiting_approval
    poll_task_status(&app, &task_id, &["awaiting_approval"]).await;

    // Get and reject approval
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/tasks/{}/approvals", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let approvals: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let approval_id = approvals[0]["id"].as_str().unwrap().to_string();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/tasks/{}/approvals/{}/action",
                    task_id, approval_id
                ))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({ "action": "reject" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    poll_task_status(&app, &task_id, &["cancelled"]).await;

    // Run eval - should fail because cancelled workflow doesn't have "completed" in traces
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/tasks/{}/eval", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        summary["failed"], 1,
        "Eval should fail on cancelled workflow: {:?}",
        summary
    );
}

#[tokio::test]
async fn test_audit_logs_record_tool_calls_and_approval_results() {
    let (app, state) = test_app().await;
    write_full_test_pack(&state.scenarios_dir, "test.audit-pack", true);

    // Run the workflow
    let body = Body::from(
        serde_json::to_string(&json!({
            "user_goal": "Execute for audit log verification"
        }))
        .unwrap(),
    );

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/test.audit-pack/run")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = response["task_id"].as_str().unwrap().to_string();

    // Wait for awaiting_approval
    poll_task_status(&app, &task_id, &["awaiting_approval"]).await;

    // Get and approve approval
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/tasks/{}/approvals", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let approvals: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let approval_id = approvals[0]["id"].as_str().unwrap().to_string();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/tasks/{}/approvals/{}/action",
                    task_id, approval_id
                ))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({ "action": "approve" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Wait for completion
    poll_task_status(&app, &task_id, &["completed"]).await;

    // Query audit logs
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/tasks/{}/audit", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let audit_logs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should have at least 2 audit entries: mock_check_service (auto) and mock_cleanup_temp (approved)
    assert!(
        audit_logs.len() >= 2,
        "Should have audit log entries: {:?}",
        audit_logs
    );

    // First entry should be mock_check_service with auto approval_status
    assert!(
        audit_logs.iter().any(|log| {
            log["action"].as_str().unwrap_or("") == "mock_check_service"
                && log["approval_status"].as_str().unwrap_or("") == "auto"
        }),
        "Should have audit entry for mock_check_service with auto result"
    );

    // Should have audit entry for mock_cleanup_temp with approved approval_status
    assert!(
        audit_logs.iter().any(|log| {
            log["action"].as_str().unwrap_or("") == "mock_cleanup_temp"
                && log["approval_status"].as_str().unwrap_or("") == "approved"
        }),
        "Should have audit entry for mock_cleanup_temp with approved result"
    );
}

/// Copy a directory recursively.
fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &dst_path);
        } else {
            fs::copy(&path, &dst_path).unwrap();
        }
    }
}

#[tokio::test]
async fn test_real_demo_ops_eval_passes() {
    // Use real demo-ops pack from the scenarios directory
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir.parent().unwrap().parent().unwrap();
    let real_pack_dir = workspace.join("scenarios/demo-ops");
    if !real_pack_dir.exists() {
        return; // skip if demo-ops pack not present
    }

    let (app, state) = test_app().await;

    // Copy real demo-ops pack into test scenarios dir (dir name doesn't matter, manifest.id is used)
    copy_dir_all(&real_pack_dir, &state.scenarios_dir.join("demo-ops"));

    // Run the workflow (pack_id = manifest.id = "demo.ops")
    let body = Body::from(
        serde_json::to_string(&json!({
            "user_goal": "Execute daily system inspection"
        }))
        .unwrap(),
    );

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/packs/demo.ops/run")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = response["task_id"].as_str().unwrap().to_string();

    // Wait for awaiting_approval (cleanup step triggers approval)
    let status = poll_task_status(&app, &task_id, &["awaiting_approval"]).await;
    assert_eq!(status, "awaiting_approval");

    // Get and approve approval
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/tasks/{}/approvals", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let approvals: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!approvals.is_empty(), "Should have pending approval");
    let approval_id = approvals[0]["id"].as_str().unwrap().to_string();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/tasks/{}/approvals/{}/action",
                    task_id, approval_id
                ))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({ "action": "approve" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Wait for completion
    let status = poll_task_status(&app, &task_id, &["completed"]).await;
    assert_eq!(status, "completed");

    // Run eval against the explicit task_id
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/tasks/{}/eval", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // All 3 eval cases should pass
    assert_eq!(
        summary["total"], 3,
        "demo-ops has 3 eval cases: {:?}",
        summary
    );
    assert_eq!(
        summary["passed"], 3,
        "All eval cases should pass: {:?}",
        summary
    );
    assert_eq!(summary["failed"], 0);
}
