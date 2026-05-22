use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::adapter::{ToolActionMeta, ToolAdapter};

/// Records a single mock tool call for inspection.
#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    pub tool_id: String,
    pub action: String,
    pub input: Value,
    pub output: Value,
    pub success: bool,
}

/// Mock tool adapter that simulates tool behavior for demo scenario packs.
pub struct MockToolAdapter {
    call_log: Arc<Mutex<Vec<ToolCallRecord>>>,
    /// Preset responses: keyed by action name.
    responses: HashMap<String, MockResponse>,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub output: Value,
    pub risk: String,           // low, medium, high
    pub default_policy: String, // allow, ask, deny
}

impl MockToolAdapter {
    pub fn new() -> Self {
        let mut responses = HashMap::new();

        responses.insert(
            "mock_check_service".to_string(),
            MockResponse {
                output: json!({ "status": "running", "services": ["api", "worker", "scheduler"] }),
                risk: "low".to_string(),
                default_policy: "allow".to_string(),
            },
        );

        responses.insert(
            "mock_check_storage".to_string(),
            MockResponse {
                output: json!({ "status": "warning", "disk_usage_percent": 85, "message": "storage warning: disk usage 85%" }),
                risk: "low".to_string(),
                default_policy: "allow".to_string(),
            },
        );

        responses.insert(
            "mock_check_network".to_string(),
            MockResponse {
                output: json!({ "status": "healthy", "latency_ms": 12 }),
                risk: "low".to_string(),
                default_policy: "allow".to_string(),
            },
        );

        responses.insert(
            "mock_cleanup_temp".to_string(),
            MockResponse {
                output: json!({ "status": "completed", "files_removed": 42, "space_freed_mb": 128 }),
                risk: "medium".to_string(),
                default_policy: "ask".to_string(),
            },
        );

        Self {
            call_log: Arc::new(Mutex::new(Vec::new())),
            responses,
        }
    }

    /// Get all recorded tool calls.
    pub async fn call_log(&self) -> Vec<ToolCallRecord> {
        self.call_log.lock().await.clone()
    }

    /// Get the mock response for an action.
    pub fn get_mock_response(&self, action: &str) -> Option<&MockResponse> {
        self.responses.get(action)
    }
}

#[async_trait]
impl ToolAdapter for MockToolAdapter {
    fn id(&self) -> &str {
        "mock"
    }

    async fn call(&self, action: &str, params: Value) -> Result<Value> {
        let response = self
            .responses
            .get(action)
            .ok_or_else(|| anyhow::anyhow!("Unknown mock action: {}", action))?;

        let output = response.output.clone();

        self.call_log.lock().await.push(ToolCallRecord {
            tool_id: "mock".to_string(),
            action: action.to_string(),
            input: params,
            output: output.clone(),
            success: true,
        });

        Ok(output)
    }

    async fn healthcheck(&self) -> bool {
        true
    }

    fn tool_metadata(&self, action: &str) -> Option<ToolActionMeta> {
        self.responses.get(action).map(|r| ToolActionMeta {
            risk: r.risk.clone(),
            default_policy: r.default_policy.clone(),
        })
    }
}
