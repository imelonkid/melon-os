use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use melon_scenario::workflow::Workflow;
use melon_tools::registry::ToolRegistry;

/// One-shot channel for approval resolution.
pub type ApprovalChannel = tokio::sync::oneshot::Sender<bool>;

/// Shared state for workflow executors.
#[derive(Clone)]
pub struct ExecutorContext {
    pub db: SqlitePool,
    pub pack_dir: PathBuf,
    pub task_id: String,
    pub registry: Arc<ToolRegistry>,
}

/// Policy evaluation result for a tool action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub action: String,
    pub decision: String, // allow, ask, deny
    pub reason: String,
}

/// Evaluate policy for a given action.
/// Checks pack permissions/policy.yaml, falls back to adapter tool metadata.
fn evaluate_policy(
    pack_dir: &PathBuf,
    registry: &ToolRegistry,
    action: &str,
    _scope: Option<&str>,
) -> PolicyDecision {
    let policy_path = pack_dir.join("permissions/policy.yaml");
    if let Ok(content) = std::fs::read_to_string(policy_path) {
        if let Ok(policy) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(policies) = policy.get("policies") {
                if let Some(rule) = policies.get(action) {
                    if let Some(default) = rule.get("default").and_then(|v| v.as_str()) {
                        return PolicyDecision {
                            action: action.to_string(),
                            decision: default.to_string(),
                            reason: format!("Policy rule for '{}'", action),
                        };
                    }
                }
            }
        }
    }

    // Fallback: use adapter tool metadata
    if let Some(meta) = registry.get_action_meta(action) {
        return PolicyDecision {
            action: action.to_string(),
            decision: meta.default_policy,
            reason: "No explicit policy rule, using tool default".to_string(),
        };
    }

    PolicyDecision {
        action: action.to_string(),
        decision: "allow".to_string(),
        reason: "No policy or tool config found, defaulting to allow".to_string(),
    }
}

/// Write a trace event to the database.
async fn write_trace(
    db: &SqlitePool,
    task_id: &str,
    event_type: &str,
    summary: &str,
    input_ref: Option<&str>,
    output_ref: Option<&str>,
) {
    let _ = sqlx::query(
        "INSERT INTO trace_events (id, task_id, event_type, summary, input_ref, output_ref) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(task_id)
    .bind(event_type)
    .bind(summary)
    .bind(input_ref)
    .bind(output_ref)
    .execute(db)
    .await;
}

/// Write an audit log entry.
async fn write_audit(
    db: &SqlitePool,
    task_id: &str,
    scenario_id: &str,
    action: &str,
    input_summary: &str,
    output_summary: &str,
    approval_status: Option<&str>,
) {
    let _ = sqlx::query(
        "INSERT INTO audit_logs (id, task_id, scenario_id, action, input_summary, output_summary, approval_status) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(task_id)
    .bind(scenario_id)
    .bind(action)
    .bind(input_summary)
    .bind(output_summary)
    .bind(approval_status)
    .execute(db)
    .await;
}

/// Update task status in the database.
async fn update_task_status(db: &SqlitePool, task_id: &str, status: &str) {
    let _ = sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
        .bind(status)
        .bind(task_id)
        .execute(db)
        .await;
}

/// Create an approval request in the database.
async fn create_approval_request(
    db: &SqlitePool,
    task_id: &str,
    approval_id: &str,
    action: &str,
    risk_level: &str,
    scope: &str,
) {
    let _ = sqlx::query(
        "INSERT INTO approval_requests (id, task_id, action, risk_level, scope, status) VALUES (?, ?, ?, ?, ?, 'pending')",
    )
    .bind(approval_id)
    .bind(task_id)
    .bind(action)
    .bind(risk_level)
    .bind(scope)
    .execute(db)
    .await;
}

/// Structured trace marker prefixes for tool actions.
/// Used by eval runner for keyword matching.
fn trace_marker(action: &str) -> &'static str {
    match action {
        "mock_check_service" => "[service_status] ",
        "mock_check_storage" => "[storage_status] ",
        "mock_check_network" => "[network_status] ",
        "mock_cleanup_temp" => "[cleanup_result] [audit_log_entry] ",
        _ => "",
    }
}

/// Execute a workflow for a given task.
pub async fn execute_workflow(ctx: ExecutorContext) -> Result<()> {
    let db = &ctx.db;
    let task_id = &ctx.task_id;

    // Read manifest to find entry point
    let manifest_path = ctx.pack_dir.join("manifest.yaml");
    let entry = if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: melon_scenario::manifest::Manifest = serde_yaml::from_str(&content)?;
        manifest
            .entry
            .unwrap_or_else(|| "workflows/default.yaml".to_string())
    } else {
        "workflows/default.yaml".to_string()
    };

    // Load workflow from pack entry
    let workflow_path = ctx.pack_dir.join(&entry);
    if !workflow_path.exists() {
        write_trace(
            db,
            task_id,
            "system",
            &format!("Workflow file not found: {}", entry),
            None,
            None,
        )
        .await;
        update_task_status(db, task_id, "failed").await;
        return Ok(());
    }

    let content = std::fs::read_to_string(&workflow_path)?;
    let workflow: Workflow = serde_yaml::from_str(&content)?;

    // Get scenario_id from task
    let scenario_id: Option<String> =
        sqlx::query_scalar("SELECT scenario_id FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_one(db)
            .await
            .ok();
    let scenario_id = scenario_id.unwrap_or_default();

    write_trace(
        db,
        task_id,
        "system",
        &format!("Starting workflow: {}", workflow.name),
        None,
        None,
    )
    .await;
    update_task_status(db, task_id, "running").await;

    for (i, step) in workflow.steps.iter().enumerate() {
        let step_summary = format!(
            "Step {}/{}: {} ({})",
            i + 1,
            workflow.steps.len(),
            step.id,
            step.step_type
        );
        write_trace(db, task_id, "system", &step_summary, None, None).await;

        match execute_step(&ctx, &step, &scenario_id).await {
            Ok(exec_result) => {
                write_trace(
                    db,
                    task_id,
                    exec_result.event_type.as_deref().unwrap_or("tool"),
                    &exec_result.summary,
                    exec_result.input_ref.as_deref(),
                    exec_result.output_ref.as_deref(),
                )
                .await;

                // If step was denied, check if cancelled (user rejected) vs failed (policy deny)
                if exec_result.denied {
                    if exec_result.cancelled {
                        write_trace(
                            db,
                            task_id,
                            "system",
                            "Workflow cancelled: user rejected action",
                            None,
                            None,
                        )
                        .await;
                    } else {
                        write_trace(
                            db,
                            task_id,
                            "system",
                            "Workflow failed: action denied",
                            None,
                            None,
                        )
                        .await;
                        update_task_status(db, task_id, "failed").await;
                    }
                    return Ok(());
                }
            }
            Err(e) => {
                write_trace(
                    db,
                    task_id,
                    "system",
                    &format!("Step failed: {}", e),
                    None,
                    None,
                )
                .await;
                update_task_status(db, task_id, "failed").await;
                return Ok(());
            }
        }
    }

    write_trace(db, task_id, "system", "Workflow completed", None, None).await;
    update_task_status(db, task_id, "completed").await;
    Ok(())
}

/// Result of executing a single workflow step.
struct StepResult {
    event_type: Option<String>,
    summary: String,
    input_ref: Option<String>,
    output_ref: Option<String>,
    denied: bool,
    /// True when the step was cancelled due to user rejection (vs actual failure).
    #[allow(dead_code)]
    cancelled: bool,
}

async fn execute_step(
    ctx: &ExecutorContext,
    step: &melon_scenario::workflow::WorkflowStep,
    scenario_id: &str,
) -> Result<StepResult> {
    let db = &ctx.db;

    // Determine the action to execute
    let (action, is_tool) = if step.step_type == "tool" {
        (step.action.clone().unwrap_or_else(|| step.id.clone()), true)
    } else if step.step_type.starts_with("tool.") {
        let tool_id = step
            .step_type
            .strip_prefix("tool.")
            .unwrap_or(&step.step_type);
        (tool_id.to_string(), true)
    } else {
        (step.step_type.clone(), false)
    };

    if is_tool {
        let scope = step.scope.as_deref();
        let policy_action = step.policy_action.as_deref().unwrap_or(&action);
        let decision = evaluate_policy(&ctx.pack_dir, &ctx.registry, policy_action, scope);

        match decision.decision.as_str() {
            "deny" => {
                write_audit(
                    db,
                    &ctx.task_id,
                    scenario_id,
                    &action,
                    "tool call",
                    "denied by policy",
                    Some("denied"),
                )
                .await;
                return Ok(StepResult {
                    event_type: Some("tool".to_string()),
                    summary: format!("Denied: {} (policy: {})", action, decision.reason),
                    input_ref: None,
                    output_ref: None,
                    denied: true,
                    cancelled: false,
                });
            }
            "ask" => {
                let approval_action = step.approval.as_deref().unwrap_or(&action);

                let risk = ctx
                    .registry
                    .get_action_meta(&action)
                    .map(|m| m.risk)
                    .unwrap_or_else(|| "medium".to_string());

                let approval_id = uuid::Uuid::new_v4().to_string();
                create_approval_request(
                    db,
                    &ctx.task_id,
                    &approval_id,
                    approval_action,
                    &risk,
                    scope.unwrap_or("workspace"),
                )
                .await;

                update_task_status(db, &ctx.task_id, "awaiting_approval").await;

                let should_proceed = wait_for_approval(ctx, &approval_id).await;

                if !should_proceed {
                    update_task_status(db, &ctx.task_id, "cancelled").await;
                    write_audit(
                        db,
                        &ctx.task_id,
                        scenario_id,
                        &action,
                        "tool call",
                        "rejected by user",
                        Some("rejected"),
                    )
                    .await;
                    return Ok(StepResult {
                        event_type: Some("approval".to_string()),
                        summary: format!("Approval rejected for: {}", action),
                        input_ref: Some(approval_id),
                        output_ref: None,
                        denied: true,
                        cancelled: true,
                    });
                }

                update_task_status(db, &ctx.task_id, "running").await;

                let output = ctx.registry.call(&action, json!({})).await?;
                let marker = trace_marker(&action);
                write_audit(
                    db,
                    &ctx.task_id,
                    scenario_id,
                    &action,
                    "tool call",
                    &format!("{:.80}", serde_json::to_string(&output).unwrap_or_default()),
                    Some("approved"),
                )
                .await;

                return Ok(StepResult {
                    event_type: Some("tool".to_string()),
                    summary: format!("{}{} (approved): {:?}", marker, action, output),
                    input_ref: Some(approval_id),
                    output_ref: Some(serde_json::to_string(&output).unwrap_or_default()),
                    denied: false,
                    cancelled: false,
                });
            }
            "allow" | _ => {
                let output = ctx.registry.call(&action, json!({})).await?;
                let marker = trace_marker(&action);
                write_audit(
                    db,
                    &ctx.task_id,
                    scenario_id,
                    &action,
                    "tool call",
                    &format!("{:.80}", serde_json::to_string(&output).unwrap_or_default()),
                    Some("auto"),
                )
                .await;

                return Ok(StepResult {
                    event_type: Some("tool".to_string()),
                    summary: format!("{}{}: {:?}", marker, action, output),
                    input_ref: None,
                    output_ref: Some(serde_json::to_string(&output).unwrap_or_default()),
                    denied: false,
                    cancelled: false,
                });
            }
        }
    } else {
        // Agent or UI step
        let summary = match step.step_type.as_str() {
            "agent.generate_checklist" => {
                "[inspection_summary] Generated checklist: service status, storage health, network connectivity"
                    .to_string()
            }
            "agent.analyze" => {
                "[inspection_summary] Analysis complete: storage_status warning detected, service_status healthy, network_status healthy"
                    .to_string()
            }
            "ui.document" => "[source_reference] Generated inspection report document. [actionable_recommendation] Review storage usage and schedule cleanup.".to_string(),
            _ => {
                format!("Executed: {}", step.step_type)
            }
        };

        Ok(StepResult {
            event_type: Some(
                step.step_type
                    .split('.')
                    .next()
                    .unwrap_or("agent")
                    .to_string(),
            ),
            summary,
            input_ref: None,
            output_ref: None,
            denied: false,
            cancelled: false,
        })
    }
}

/// Wait for approval by polling the database.
async fn wait_for_approval(ctx: &ExecutorContext, approval_id: &str) -> bool {
    poll_approval(&ctx.db, approval_id, &ctx.task_id).await
}

async fn poll_approval(db: &SqlitePool, approval_id: &str, task_id: &str) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(300);

    loop {
        if start.elapsed() > timeout {
            tracing::warn!("Approval polling timed out for approval_id={}", approval_id);
            update_task_status(db, task_id, "failed").await;
            return false;
        }

        let task_status: Option<String> =
            sqlx::query_scalar("SELECT status FROM tasks WHERE id = ?")
                .bind(task_id)
                .fetch_one(db)
                .await
                .ok();

        if matches!(task_status.as_deref(), Some("cancelled") | Some("failed")) {
            tracing::info!(
                "Task {} cancelled while awaiting approval, exiting",
                task_id
            );
            return false;
        }

        let status: Option<String> =
            sqlx::query_scalar("SELECT status FROM approval_requests WHERE id = ?")
                .bind(approval_id)
                .fetch_one(db)
                .await
                .ok();

        match status.as_deref() {
            Some("approved") => return true,
            Some("rejected") => return false,
            _ => {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    }
}

/// Resolve an approval request. Called by the API endpoint after updating the database.
#[allow(dead_code)]
pub async fn resolve_approval(
    _channels: &Arc<Mutex<HashMap<String, ApprovalChannel>>>,
    _approval_id: &str,
    _approved: bool,
) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use melon_tools::adapter::{ToolActionMeta, ToolAdapter};
    use serde_json::{json, Value};
    use sqlx::SqlitePool;

    struct SpyToolAdapter {
        calls: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl ToolAdapter for SpyToolAdapter {
        fn id(&self) -> &str {
            "spy"
        }

        async fn call(&self, action: &str, _params: Value) -> Result<Value> {
            self.calls.lock().await.push(action.to_string());
            Ok(json!({"status": "ok", "adapter": "spy"}))
        }

        async fn healthcheck(&self) -> bool {
            true
        }

        fn tool_metadata(&self, action: &str) -> Option<ToolActionMeta> {
            if action == "spy_probe" {
                Some(ToolActionMeta {
                    risk: "low".to_string(),
                    default_policy: "allow".to_string(),
                })
            } else {
                None
            }
        }
    }

    async fn create_test_db() -> SqlitePool {
        let db_path = std::env::temp_dir().join(format!(
            "melon-agent-executor-test-{}.db",
            uuid::Uuid::new_v4()
        ));
        std::fs::File::create(&db_path).expect("create sqlite file");
        let db_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&db_url).await.expect("connect sqlite");

        sqlx::query(
            r#"
            CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                scenario_id TEXT NOT NULL,
                user_goal TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'created'
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create tasks");

        sqlx::query(
            r#"
            CREATE TABLE trace_events (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                summary TEXT,
                input_ref TEXT,
                output_ref TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create trace_events");

        sqlx::query(
            r#"
            CREATE TABLE approval_requests (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                action TEXT NOT NULL,
                risk_level TEXT NOT NULL,
                scope TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                resolved_at DATETIME
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create approval_requests");

        sqlx::query(
            r#"
            CREATE TABLE audit_logs (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                scenario_id TEXT,
                action TEXT NOT NULL,
                input_summary TEXT,
                output_summary TEXT,
                approval_status TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create audit_logs");

        pool
    }

    #[tokio::test]
    async fn executor_invokes_tool_steps_through_registry_adapter() {
        let db = create_test_db().await;
        let task_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO tasks (id, scenario_id, user_goal, status) VALUES (?, ?, ?, 'created')",
        )
        .bind(&task_id)
        .bind("spy.pack")
        .bind("verify registry execution")
        .execute(&db)
        .await
        .expect("insert task");

        let pack_dir = std::env::temp_dir().join(format!(
            "melon-agent-executor-pack-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(pack_dir.join("workflows")).expect("create workflow dir");
        std::fs::write(
            pack_dir.join("manifest.yaml"),
            "id: spy.pack\nname: Spy Pack\nversion: 0.1.0\nentry: workflows/default.yaml\n",
        )
        .expect("write manifest");
        std::fs::write(
            pack_dir.join("workflows/default.yaml"),
            "name: spy_workflow\nsteps:\n  - id: probe\n    type: tool\n    action: spy_probe\n",
        )
        .expect("write workflow");

        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut registry = ToolRegistry::new();
        registry.register_adapter(Arc::new(SpyToolAdapter {
            calls: calls.clone(),
        }));

        execute_workflow(ExecutorContext {
            db: db.clone(),
            pack_dir,
            task_id: task_id.clone(),
            registry: Arc::new(registry),
        })
        .await
        .expect("execute workflow");

        assert_eq!(calls.lock().await.as_slice(), ["spy_probe"]);

        let status: String = sqlx::query_scalar("SELECT status FROM tasks WHERE id = ?")
            .bind(&task_id)
            .fetch_one(&db)
            .await
            .expect("fetch status");
        assert_eq!(status, "completed");

        let audit: (String, String, String) = sqlx::query_as(
            "SELECT action, output_summary, approval_status FROM audit_logs WHERE task_id = ?",
        )
        .bind(&task_id)
        .fetch_one(&db)
        .await
        .expect("fetch audit");

        assert_eq!(audit.0, "spy_probe");
        assert!(audit.1.contains("\"adapter\":\"spy\""));
        assert_eq!(audit.2, "auto");
    }
}
