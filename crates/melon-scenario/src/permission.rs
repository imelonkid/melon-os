use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PermissionPolicy {
    pub policies: std::collections::HashMap<String, PolicyRule>,
    pub audit: Option<AuditConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyRule {
    pub default: String, // allow, ask, deny, allow_once, allow_session
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditConfig {
    pub enabled: bool,
    pub retain_days: Option<u32>,
}
