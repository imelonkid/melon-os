use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Created,
    Planning,
    AwaitingApproval,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub scenario_id: String,
    pub user_goal: String,
    pub status: TaskStatus,
}

impl Task {
    pub fn new(scenario_id: String, user_goal: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            scenario_id,
            user_goal,
            status: TaskStatus::Created,
        }
    }
}
