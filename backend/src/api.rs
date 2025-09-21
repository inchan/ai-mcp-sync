use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::config::{
    ImportRecommendedRequest, MasterConfigResponse, McpSettings, RecommendedServer, SyncRequest,
    SyncSummary, ToolConfiguration, UpdateMasterRequest,
};
use crate::db::Database;
use crate::error::{BackendError, BackendResult};
use crate::sync;

#[derive(Clone)]
pub struct AppState {
    pub db: std::sync::Arc<Database>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/tools", get(list_tools))
        .route("/api/tools/rescan", post(rescan_tools))
        .route(
            "/api/config/master",
            get(get_master_config).post(update_master_config),
        )
        .route("/api/config/recommended", get(get_recommended_servers))
        .route("/api/config/master/import", post(import_recommended_server))
        .route("/api/sync", post(sync_tools))
        .route("/api/sync/history", get(sync_history))
        .with_state(state)
}

async fn list_tools(State(state): State<AppState>) -> BackendResult<Json<Vec<ToolConfiguration>>> {
    let tools = state
        .db
        .list_tools()?
        .into_iter()
        .map(|(name, path)| {
            let settings =
                sync::read_settings_from_file(&path).unwrap_or_else(|_| McpSettings::default());
            ToolConfiguration::new(name, path.to_string_lossy(), settings)
        })
        .collect();
    Ok(Json(tools))
}

async fn rescan_tools(
    State(state): State<AppState>,
) -> BackendResult<Json<Vec<ToolConfiguration>>> {
    let tools = sync::discover_tools(&state.db)?;
    Ok(Json(tools))
}

async fn get_master_config(
    State(state): State<AppState>,
) -> BackendResult<Json<MasterConfigResponse>> {
    let config = state.db.ensure_master_config()?;
    Ok(Json(config))
}

async fn update_master_config(
    State(state): State<AppState>,
    Json(payload): Json<UpdateMasterRequest>,
) -> BackendResult<Json<MasterConfigResponse>> {
    state.db.upsert_master_config(&payload.settings)?;
    let config = state.db.ensure_master_config()?;
    Ok(Json(config))
}

async fn get_recommended_servers(
    State(state): State<AppState>,
) -> BackendResult<Json<Vec<RecommendedServer>>> {
    let servers = state.db.list_recommended_servers()?;
    Ok(Json(servers))
}

async fn import_recommended_server(
    State(state): State<AppState>,
    Json(payload): Json<ImportRecommendedRequest>,
) -> BackendResult<Json<MasterConfigResponse>> {
    let server = state
        .db
        .get_recommended_server(&payload.server_id)?
        .ok_or_else(|| {
            BackendError::NotFound(format!("recommended server '{}'", payload.server_id))
        })?;

    let mut master = state.db.ensure_master_config()?.settings;
    let enabled = payload.enabled.unwrap_or(server.default_enabled);
    master.apply_recommended_server(&server, enabled);

    state.db.upsert_master_config(&master)?;
    let updated = state.db.ensure_master_config()?;
    Ok(Json(updated))
}

async fn sync_tools(
    State(state): State<AppState>,
    Json(request): Json<SyncRequest>,
) -> BackendResult<Json<Vec<SyncSummary>>> {
    let master = state.db.ensure_master_config()?.settings;
    let tools = if let Some(tool_name) = request.tool {
        state
            .db
            .list_tools()?
            .into_iter()
            .filter(|(name, _)| *name == tool_name)
            .collect::<Vec<_>>()
    } else {
        state.db.list_tools()?
    };

    let mut summaries = Vec::new();
    for (name, path) in tools {
        let settings = sync::read_settings_from_file(&path)?;
        let tool_config = ToolConfiguration::new(name.clone(), path.to_string_lossy(), settings);
        let summary = sync::sync_tool(&tool_config, &master, &state.db)?;
        summaries.push(summary);
    }
    Ok(Json(summaries))
}

async fn sync_history(State(state): State<AppState>) -> BackendResult<Json<Vec<SyncSummary>>> {
    let history = state.db.recent_sync_history(25)?;
    Ok(Json(history))
}
