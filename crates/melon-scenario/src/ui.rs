use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiLayout {
    pub layout: Option<LayoutConfig>,
    pub views: Vec<UiView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LayoutConfig {
    pub default: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiView {
    pub id: String,
    #[serde(rename = "type")]
    pub view_type: String, // chat, document, table, kanban, task_graph, device_panel, approval
    pub region: Option<String>, // left, main, right, bottom
}
