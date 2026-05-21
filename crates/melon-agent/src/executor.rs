use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use melon_scenario::workflow::Workflow;

/// One-shot channel for approval resolution.
pub type ApprovalChannel = tokio::sync::oneshot::Sender<bool>;

/// Shared state for workflow executors.
#[derive(Clone)]
pub struct ExecutorContext {
    pub db: SqlitePool,
    pub pack_dir: PathBuf,
    pub task_id: String,
    /// Approval channels: approval_id -> oneshot sender.
    pub approval_channels: Arc<Mutex<HashMap<String, ApprovalChannel>>>,
}

/// Policy evaluation result for a tool action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub action: String,
    pub decision: String, // allow, ask, deny
    pub reason: String,
}

/// Mock tool response data.
#[derive(Debug, Clone)]
pub struct MockToolResponse {
    pub output: serde_json::Value,
    pub risk: String,
    pub default_policy: String,
}

/// Build mock tool responses matching the demo-ops pack.
fn mock_tool_response(action: &str) -> Option<MockToolResponse> {
    match action {
        "mock_check_service" => Some(MockToolResponse {
            output: json!({"status": "running", "services": ["api", "worker", "scheduler"]}),
            risk: "low".to_string(),
            default_policy: "allow".to_string(),
        }),
        "mock_check_storage" => Some(MockToolResponse {
            output: json!({"status": "warning", "disk_usage_percent": 85, "message": "storage warning: disk usage 85%"}),
            risk: "low".to_string(),
            default_policy: "allow".to_string(),
        }),
        "mock_check_network" => Some(MockToolResponse {
            output: json!({"status": "healthy", "latency_ms": 12}),
            risk: "low".to_string(),
            default_policy: "allow".to_string(),
        }),
        "mock_cleanup_temp" => Some(MockToolResponse {
            output: json!({"status": "completed", "files_removed": 42, "space_freed_mb": 128}),
            risk: "medium".to_string(),
            default_policy: "ask".to_string(),
        }),
        _ => None,
    }
}

/// Evaluate policy for a given action.
/// For MVP: checks pack permissions/policy.yaml, falls back to mock tool default_policy.
fn evaluate_policy(pack_dir: &PathBuf, action: &str, _scope: Option<&str>) -> PolicyDecision {
    let policy_path = pack_dir.join("permissions/policy.yaml");
    if let Ok(content) = std::fs::read_to_string(policy_path) {
        if let Ok(policy) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(policies) = policy.get("policies") {
                // Check if there's a rule for this action
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

    // Fallback: use mock tool default_policy
    if let Some(mock) = mock_tool_response(action) {
        return PolicyDecision {
            action: action.to_string(),
            decision: mock.default_policy,
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
                        // Status already set to cancelled by the step handler
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
    // step_type can be "tool" (with separate action field) or "tool.{id}" or "agent.{action}" or "ui.{type}"
    let (action, is_tool) = if step.step_type == "tool" {
        // New format: type: tool, action: mock_xxx
        (step.action.clone().unwrap_or_else(|| step.id.clone()), true)
    } else if step.step_type.starts_with("tool.") {
        // Legacy format: type: tool.mock_xxx
        let tool_id = step
            .step_type
            .strip_prefix("tool.")
            .unwrap_or(&step.step_type);
        (tool_id.to_string(), true)
    } else {
        // Agent or UI step - use step_type as action identifier
        (step.step_type.clone(), false)
    };

    if is_tool {
        // Check policy
        let scope = step.scope.as_deref();
        let policy_action = step.policy_action.as_deref().unwrap_or(&action);
        let decision = evaluate_policy(&ctx.pack_dir, policy_action, scope);

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
                // Check if step.approval is set (user wants approval for this step)
                let approval_action = step.approval.as_deref().unwrap_or(&action);

                let mock = mock_tool_response(&action);
                let risk = mock.as_ref().map(|m| m.risk.as_str()).unwrap_or("medium");

                let approval_id = uuid::Uuid::new_v4().to_string();
                create_approval_request(
                    db,
                    &ctx.task_id,
                    &approval_id,
                    approval_action,
                    risk,
                    scope.unwrap_or("workspace"),
                )
                .await;

                // Transition to awaiting_approval so UI can show pending state
                update_task_status(db, &ctx.task_id, "awaiting_approval").await;

                // Wait for user to approve/reject
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

                // Resume running after approval
                update_task_status(db, &ctx.task_id, "running").await;

                // Execute the tool after approval
                let output = execute_mock_tool(&action).await?;
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
                    summary: format!("{} (approved): {:?}", action, output),
                    input_ref: Some(approval_id),
                    output_ref: Some(serde_json::to_string(&output).unwrap_or_default()),
                    denied: false,
                    cancelled: false,
                });
            }
            "allow" | _ => {
                let output = execute_mock_tool(&action).await?;
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
                    summary: format!("{}: {:?}", action, output),
                    input_ref: None,
                    output_ref: Some(serde_json::to_string(&output).unwrap_or_default()),
                    denied: false,
                    cancelled: false,
                });
            }
        }
    } else {
        // Agent or UI step - simulate
        let summary = match step.step_type.as_str() {
            "agent.generate_checklist" => {
                "Generated checklist: service status, storage health, network connectivity"
                    .to_string()
            }
            "agent.analyze" => {
                "Analysis complete: storage warning detected, other services healthy".to_string()
            }
            "ui.document" => "Generated inspection report document".to_string(),
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

/// Execute a mock tool and return its output.
async fn execute_mock_tool(action: &str) -> Result<serde_json::Value> {
    let mock = mock_tool_response(action)
        .ok_or_else(|| anyhow::anyhow!("Unknown mock tool: {}", action))?;
    Ok(mock.output)
}

/// Wait for approval using oneshot channel, or poll the database.
async fn wait_for_approval(ctx: &ExecutorContext, approval_id: &str) -> bool {
    // Try to use the in-memory channel first
    {
        let mut channels = ctx.approval_channels.lock().await;
        if let Some((_, _tx)) = channels.remove_entry(approval_id) {
            // Channel exists - the approval endpoint will resolve it
            // We need to actually wait - so we create a new oneshot pair
            drop(channels);
            let (tx, rx) = tokio::sync::oneshot::channel();
            ctx.approval_channels
                .lock()
                .await
                .insert(approval_id.to_string(), tx);
            // Wait for the channel
            match rx.await {
                Ok(approved) => return approved,
                Err(_) => {
                    // Channel closed, fall back to polling
                    return poll_approval(&ctx.db, approval_id).await;
                }
            }
        }
    }

    // No in-memory channel - this means the executor is spawned as a separate task
    // We need to signal the external waiter that we're waiting
    // For MVP, just poll the database
    poll_approval(&ctx.db, approval_id).await
}

async fn poll_approval(db: &SqlitePool, approval_id: &str) -> bool {
    loop {
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

/// Resolve an approval request. This is called by the API endpoint.
/// If the executor is waiting via a channel, it will be notified immediately.
/// Otherwise, the executor will pick up the change via polling.
pub async fn resolve_approval(
    channels: &Arc<Mutex<HashMap<String, ApprovalChannel>>>,
    approval_id: &str,
    approved: bool,
) -> bool {
    // Try in-memory channel first
    let mut map = channels.lock().await;
    if let Some(tx) = map.remove(approval_id) {
        let _ = tx.send(approved);
        return true;
    }
    // Otherwise the database update will be picked up by polling
    false
}
