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
        .merge(melon_runtime::routes::files::router())
        .merge(melon_runtime::routes::validate::router())
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
