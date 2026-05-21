use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeSource {
    pub id: String,
    pub uri: String,
    #[serde(rename = "type")]
    pub source_type: String, // file, directory, web
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeSources {
    pub sources: Vec<KnowledgeSource>,
}
