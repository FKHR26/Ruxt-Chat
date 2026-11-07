use crate::model::{CreateChannelReq, CreateServerReq};
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn list_servers(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let servers = state.registry.list_servers().await;
    Json(serde_json::json!(servers))
}

pub async fn create_server(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateServerReq>,
) -> Json<serde_json::Value> {
    let server = state.registry.add_server(req.name).await;
    Json(serde_json::json!(server))
}

pub async fn create_channel(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<Uuid>,
    Json(req): Json<CreateChannelReq>,
) -> Json<serde_json::Value> {
    match state.registry.add_channel(server_id, req.name).await {
        Some(channel) => Json(serde_json::json!(channel)),
        None => Json(serde_json::json!({ "error": "server not found" })),
    }
}

pub async fn channel_history(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let history = state.registry.load_history(channel_id).await;
    Json(serde_json::json!(history))
}
