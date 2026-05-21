use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use melon_runtime::AppState;

async fn make_state(scenarios_dir: PathBuf) -> (AppState, PathBuf) {
    let db_path = format!("/tmp/melon_test_{}.db", std::process::id());
    let _ = std::fs::remove_file(&db_path);

    let pool = melon_runtime::db::init_db(&db_path).await.unwrap();

    let state = AppState {
        db: pool,
        scenarios_dir,
        approval_channels: Arc::new(Mutex::new(HashMap::new())),
    };

    (state, PathBuf::from(db_path))
}

#[tokio::test]
async fn test_discover_packs() {
    let scenarios_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("scenarios");

    let packs = melon_scenario::pack::discover_packs(&scenarios_dir).unwrap();
    assert!(!packs.is_empty(), "Should discover at least one pack");

    let pack_ids: Vec<_> = packs
        .iter()
        .map(|p| {
            let content = std::fs::read_to_string(p.join("manifest.yaml")).unwrap();
            let manifest: melon_scenario::manifest::Manifest =
                serde_yaml::from_str(&content).unwrap();
            manifest.id
        })
        .collect();

    assert!(
        pack_ids.contains(&"demo.ops".to_string()),
        "Should find demo.ops"
    );
    assert!(
        pack_ids.contains(&"melon.home".to_string()),
        "Should find melon.home"
    );
}

#[tokio::test]
async fn test_validate_demo_ops() {
    let scenarios_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("scenarios");

    let pack_dir = scenarios_dir.join("demo-ops");
    let errors = melon_scenario::validation::validate_pack(&pack_dir).unwrap();
    assert!(errors.is_empty(), "demo.ops should be valid: {:?}", errors);
}

#[tokio::test]
async fn test_validate_missing_workflow() {
    let tmp_dir = std::env::temp_dir().join(format!("melon_test_pack_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).unwrap();

    // Create manifest with entry pointing to non-existent workflow
    std::fs::write(
        tmp_dir.join("manifest.yaml"),
        "id: test.invalid\nname: Test\nversion: 0.1.0\nentry: workflows/missing.yaml\n",
    )
    .unwrap();
    std::fs::write(tmp_dir.join("role.md"), "# Test role").unwrap();

    let errors = melon_scenario::validation::validate_pack(&tmp_dir).unwrap();
    assert!(
        errors.iter().any(|e| e.contains("missing.yaml")),
        "Should report missing workflow file: {:?}",
        errors
    );

    let _ = std::fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_safe_join_rejects_traversal() {
    let base = PathBuf::from("/tmp/melon_safe_test");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();

    // Create a file inside the base
    std::fs::write(base.join("test.txt"), "hello").unwrap();

    // Valid path should work
    let result = safe_join(&base, "test.txt");
    assert!(result.is_ok(), "Valid path should work");

    // Path traversal should be blocked
    let result = safe_join(&base, "../../../etc/passwd");
    assert!(result.is_err(), "Path traversal should be blocked");

    let _ = std::fs::remove_dir_all(base);
}

fn safe_join(base: &PathBuf, relative: &str) -> Result<PathBuf, ()> {
    let full_path = base.join(relative);
    let canonical_base = base.canonicalize().map_err(|_| ())?;
    let canonical_full = full_path.canonicalize().unwrap_or_else(|_| {
        let mut result = base.clone();
        for component in PathBuf::from(relative).components() {
            if component.as_os_str() == ".." {
                result.pop();
            } else if let Some(c) = component.as_os_str().to_str() {
                result.push(c);
            }
        }
        result
    });

    if !canonical_full.starts_with(&canonical_base) {
        return Err(());
    }

    Ok(full_path)
}

#[tokio::test]
async fn test_task_persistence() {
    let scenarios_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("scenarios");

    let (state, db_path) = make_state(scenarios_dir).await;

    // Create task
    let task_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO tasks (id, scenario_id, user_goal, status) VALUES (?, ?, ?, 'created')",
    )
    .bind(&task_id)
    .bind("demo.ops")
    .bind("Test task")
    .execute(&state.db)
    .await
    .unwrap();

    // Query task
    let row: (String, String, String, String) =
        sqlx::query_as("SELECT id, scenario_id, user_goal, status FROM tasks WHERE id = ?")
            .bind(&task_id)
            .fetch_one(&state.db)
            .await
            .unwrap();

    assert_eq!(row.0, task_id);
    assert_eq!(row.1, "demo.ops");
    assert_eq!(row.2, "Test task");
    assert_eq!(row.3, "created");

    let _ = std::fs::remove_file(db_path);
}
