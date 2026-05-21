use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Top-level manifest for a scenario pack.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Manifest {
    /// Unique identifier, e.g. "melon.research"
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Short description
    pub description: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Required runtime version, e.g. ">=0.1.0"
    #[serde(rename = "runtime")]
    pub runtime_version: Option<String>,
    /// Entry workflow file, e.g. "workflows/default.yaml"
    pub entry: Option<String>,
    /// Declared permissions
    pub permissions: Option<Vec<String>>,
    /// Dependencies on MCP servers and skills
    pub dependencies: Option<Dependencies>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Dependencies {
    #[serde(default)]
    pub mcp: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            runtime_version: Some(">=0.1.0".to_string()),
            entry: Some("workflows/default.yaml".to_string()),
            permissions: None,
            dependencies: None,
        }
    }
}
