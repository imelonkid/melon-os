use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub id: String,
    pub tool_type: String,
    pub config: serde_json::Value,
}

pub struct ToolRegistry {
    tools: HashMap<String, ToolEntry>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: ToolEntry) {
        self.tools.insert(tool.id.clone(), tool);
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
}
