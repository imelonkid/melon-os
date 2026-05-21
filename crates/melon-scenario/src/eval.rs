use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvalCase {
    pub id: String,
    pub goal: String,
    pub expected: ExpectedResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExpectedResult {
    #[serde(default)]
    pub must_include: Vec<String>,
    #[serde(default)]
    pub must_not: Vec<String>,
}
