use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Metadata for a specific tool action, used by policy evaluation and risk assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolActionMeta {
    pub risk: String,           // low, medium, high
    pub default_policy: String, // allow, ask, deny
}

/// Adapter interface for tool execution.
#[async_trait::async_trait]
pub trait ToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    async fn call(&self, action: &str, params: Value) -> Result<Value>;
    async fn healthcheck(&self) -> bool;

    /// Return metadata for a specific action, if available.
    /// Used for policy fallback and risk display.
    fn tool_metadata(&self, action: &str) -> Option<ToolActionMeta> {
        let _ = action;
        None
    }
}
