use std::collections::HashMap;
use std::sync::Arc;

use super::adapter::ToolAdapter;
use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub id: String,
    pub tool_type: String,
    pub config: serde_json::Value,
}

pub struct ToolRegistry {
    tools: HashMap<String, ToolEntry>,
    adapters: HashMap<String, Arc<dyn ToolAdapter>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            adapters: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: ToolEntry) {
        self.tools.insert(tool.id.clone(), tool);
    }

    pub fn register_adapter(&mut self, adapter: Arc<dyn ToolAdapter>) {
        self.adapters.insert(adapter.id().to_string(), adapter);
    }

    pub fn get(&self, id: &str) -> Option<&ToolEntry> {
        self.tools.get(id)
    }

    pub fn list(&self) -> Vec<&ToolEntry> {
        self.tools.values().collect()
    }

    pub fn remove(&mut self, id: &str) -> Option<ToolEntry> {
        self.tools.remove(id)
    }

    /// Call a tool action via the appropriate adapter.
    /// Resolves adapter by deriving tool_id from action name:
    /// e.g. "mock_check_service" → looks for adapter "mock"
    pub async fn call(&self, action: &str, params: Value) -> Result<Value> {
        let tool_id = resolve_tool_id(action)?;
        let adapter = self
            .adapters
            .get(&tool_id)
            .ok_or_else(|| anyhow::anyhow!("No adapter registered for tool: {}", tool_id))?;
        adapter.call(action, params).await
    }

    /// Get action metadata (risk, default_policy) from the adapter.
    pub fn get_action_meta(&self, action: &str) -> Option<super::adapter::ToolActionMeta> {
        let tool_id = resolve_tool_id(action).ok()?;
        let adapter = self.adapters.get(&tool_id)?;
        adapter.tool_metadata(action)
    }

    /// List all registered adapter IDs.
    pub fn list_adapters(&self) -> Vec<&str> {
        self.adapters.keys().map(|k| k.as_str()).collect()
    }
}

/// Derive tool adapter ID from an action name.
/// Convention: "mock_check_service" → "mock", "github_create_pr" → "github"
fn resolve_tool_id(action: &str) -> Result<String> {
    let parts: Vec<&str> = action.splitn(2, '_').collect();
    if parts.len() < 2 || parts[0].is_empty() {
        return Err(anyhow::anyhow!(
            "Cannot derive tool_id from action '{}'. Expected format: <tool_id>_<action>",
            action
        ));
    }
    Ok(parts[0].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_adapter::MockToolAdapter;
    use serde_json::json;

    #[tokio::test]
    async fn registry_calls_registered_mock_adapter_and_exposes_metadata() {
        let adapter = Arc::new(MockToolAdapter::new());
        let mut registry = ToolRegistry::new();
        registry.register_adapter(adapter.clone());

        let output = registry
            .call("mock_check_service", json!({"request_id": "test-call"}))
            .await
            .expect("registered mock adapter should execute action");

        assert_eq!(output["status"], "running");
        assert_eq!(output["services"][0], "api");

        let meta = registry
            .get_action_meta("mock_cleanup_temp")
            .expect("mock cleanup metadata should come from adapter");
        assert_eq!(meta.risk, "medium");
        assert_eq!(meta.default_policy, "ask");

        let calls = adapter.call_log().await;
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_id, "mock");
        assert_eq!(calls[0].action, "mock_check_service");
        assert_eq!(calls[0].input["request_id"], "test-call");
        assert!(calls[0].success);
    }

    #[tokio::test]
    async fn registry_errors_when_no_adapter_matches_action_prefix() {
        let registry = ToolRegistry::new();

        let err = registry
            .call("mock_check_service", json!({}))
            .await
            .expect_err("unregistered mock adapter should fail");

        assert!(
            err.to_string()
                .contains("No adapter registered for tool: mock"),
            "unexpected error: {err}"
        );
    }
}
