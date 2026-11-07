use crate::auth::user_for_token;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TokenQuery {
    pub token: Uuid,
}

#[derive(Deserialize)]
pub struct AddFriendReq {
    pub token: Uuid,
    pub username: String,
}

pub async fn list_friends(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TokenQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (user_id, _) = user_for_token(&state, query.token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let friends = state.registry.list_friends(user_id).await;
    Ok(Json(serde_json::json!(friends)))
}

pub async fn add_friend(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddFriendReq>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (user_id, _) = user_for_token(&state, req.token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match state.registry.add_friend(user_id, &req.username).await {
        Some(entry) => Ok(Json(serde_json::json!(entry))),
        None => Ok(Json(serde_json::json!({ "error": "no user with that username" }))),
    }
}

pub async fn conversation_history(
    State(state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<TokenQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (user_id, _) = user_for_token(&state, query.token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !state.registry.user_in_conversation(conversation_id, user_id).await {
        return Err(StatusCode::FORBIDDEN);
    }

    let history = state.registry.load_history(conversation_id).await;
    Ok(Json(serde_json::json!(history)))
}