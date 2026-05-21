use anyhow::Result;
use serde_json::Value;

/// Adapter interface for tool execution.
#[async_trait::async_trait]
pub trait ToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    async fn call(&self, action: &str, params: Value) -> Result<Value>;
    async fn healthcheck(&self) -> bool;
}
