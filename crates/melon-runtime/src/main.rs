use anyhow::Result;
use axum::{routing::get, Json, Router};
use melon_runtime::AppState;
use serde::Serialize;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn run_server(addr: SocketAddr, state: AppState) -> Result<()> {
    let app = Router::new()
        .route("/api/health", get(health))
        .merge(melon_runtime::routes::packs::router())
        .merge(melon_runtime::routes::tasks::router())
        .merge(melon_runtime::routes::tasks_detail::router())
        .merge(melon_runtime::routes::files::router())
        .merge(melon_runtime::routes::validate::router())
        .merge(melon_runtime::routes::run::router())
        .with_state(state);

    info!("melonOS Runtime daemon starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("melon_runtime=info".parse()?))
        .init();

    info!("Starting melonOS Runtime v{}", env!("CARGO_PKG_VERSION"));

    // Resolve scenarios directory
    let scenarios_dir = std::env::var("MELON_SCENARIOS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop();
            p.pop();
            p.push("scenarios");
            p
        });

    info!("Scenarios directory: {}", scenarios_dir.display());

    // Initialize database
    let db_path = std::env::var("MELON_DB_PATH").unwrap_or_else(|_| {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
        p.pop();
        p.push("data");
        p.push("melon.db");
        p.to_string_lossy().to_string()
    });

    let pool = melon_runtime::db::init_db(&db_path)
        .await
        .expect("Failed to initialize database");

    let state = AppState {
        db: pool,
        scenarios_dir,
        approval_channels: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
    };

    let addr: SocketAddr = std::env::var("MELON_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()?;

    run_server(addr, state).await?;

    Ok(())
}
