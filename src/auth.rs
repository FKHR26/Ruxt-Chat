use crate::AppState;
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AuthReq {
    pub username: String,
    pub password: String,
}

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthReq>,
) -> Json<serde_json::Value> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(req.password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => return Json(serde_json::json!({ "error": "could not hash password" })),
    };

    let user_id = Uuid::new_v4();
    let result = sqlx::query("INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&req.username)
        .bind(&password_hash)
        .execute(&state.registry.pool)
        .await;

    if result.is_err() {
        return Json(serde_json::json!({ "error": "username already taken" }));
    }

    create_session(&state, user_id, &req.username).await
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthReq>,
) -> Json<serde_json::Value> {
    let row: Option<(Uuid, String)> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE username = $1")
            .bind(&req.username)
            .fetch_optional(&state.registry.pool)
            .await
            .unwrap_or(None);

    let Some((user_id, password_hash)) = row else {
        return Json(serde_json::json!({ "error": "invalid username or password" }));
    };

    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(h) => h,
        Err(_) => return Json(serde_json::json!({ "error": "invalid username or password" })),
    };

    if Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Json(serde_json::json!({ "error": "invalid username or password" }));
    }

    create_session(&state, user_id, &req.username).await
}

async fn create_session(state: &Arc<AppState>, user_id: Uuid, username: &str) -> Json<serde_json::Value> {
    let token = Uuid::new_v4();
    let saved = sqlx::query("INSERT INTO sessions (token, user_id) VALUES ($1, $2)")
        .bind(token)
        .bind(user_id)
        .execute(&state.registry.pool)
        .await;

    if saved.is_err() {
        return Json(serde_json::json!({ "error": "could not create session" }));
    }

    Json(serde_json::json!({ "token": token, "username": username }))
}

/// Looks up which username a session token belongs to. Returns None for
/// a missing/invalid token, which callers should treat as "not logged in".
pub async fn username_for_token(state: &Arc<AppState>, token: Uuid) -> Option<String> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT users.username FROM sessions
         JOIN users ON users.id = sessions.user_id
         WHERE sessions.token = $1",
    )
    .bind(token)
    .fetch_optional(&state.registry.pool)
    .await
    .unwrap_or(None);

    row.map(|(username,)| username)
}
