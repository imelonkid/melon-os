use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Workflow {
    pub name: String,
    #[serde(default)]
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowStep {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: String,
    /// Tool or action to invoke, e.g. "mock_check_service" or extracted from step_type
    pub action: Option<String>,
    /// Policy action name for permission check, e.g. "read_status"
    pub policy_action: Option<String>,
    /// Scope for permission check, e.g. "workspace"
    pub scope: Option<String>,
    /// If true, requires user approval before executing this step
    #[serde(default)]
    pub approval: Option<String>,
}
