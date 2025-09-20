use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use backend::api::{router, AppState};
use backend::db::Database;
use backend::error::BackendError;
use backend::sync;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), BackendError> {
    init_tracing();

    let db = Arc::new(Database::initialize()?);
    db.ensure_master_config()?;
    if let Err(err) = sync::discover_tools(&db) {
        tracing::warn!("failed to perform initial tool discovery: {err}");
    }

    let app_state = AppState { db: db.clone() };
    let app: Router = router(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| BackendError::Other(format!("failed to bind: {err}")))?;
    let actual_addr = listener
        .local_addr()
        .map_err(|err| BackendError::Other(format!("failed to read local addr: {err}")))?;
    info!(%actual_addr, "Starting MCP Sync backend");

    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|err| BackendError::Other(format!("server error: {err}")))
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .init();
}
