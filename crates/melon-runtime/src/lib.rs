use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use melon_agent::executor::ApprovalChannel;
use sqlx::SqlitePool;

pub mod db;
pub mod routes;
pub mod state;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub scenarios_dir: PathBuf,
    pub approval_channels: Arc<Mutex<HashMap<String, ApprovalChannel>>>,
}
