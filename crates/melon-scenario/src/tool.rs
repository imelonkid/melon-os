use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String, // mcp, cli, http
    pub command: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub healthcheck: Option<Healthcheck>,
    pub startup: Option<Startup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Healthcheck {
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Startup {
    pub mode: String, // on_demand, always
}
