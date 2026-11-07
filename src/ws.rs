use crate::auth::user_for_token;
use crate::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub sender_username: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: Uuid,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let Some((user_id, username)) = user_for_token(&state, query.token).await else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };

    // Make sure this user is actually a participant in this conversation,
    // so a stolen conversation id alone can't be used to read someone
    // else's DMs.
    if !state.registry.user_in_conversation(conversation_id, user_id).await {
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state, conversation_id, user_id, username))
        .into_response()
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    conversation_id: Uuid,
    user_id: Uuid,
    username: String,
) {
    let Some(tx) = state.registry.sender_for(conversation_id) else {
        return;
    };

    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let recv_tx = tx.clone();
    let registry = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let id = registry
                .registry
                .save_message(conversation_id, user_id, &text)
                .await;

            let chat_msg = ChatMessage {
                id: id.to_string(),
                sender_username: username.clone(),
                content: text,
            };
            if let Ok(json) = serde_json::to_string(&chat_msg) {
                let _ = recv_tx.send(json);
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}