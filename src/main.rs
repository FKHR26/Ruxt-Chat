mod api;
mod auth;
mod model;
mod ws;

use axum::routing::{get, post};
use axum::Router;
use model::Registry;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub struct AppState {
    pub registry: Registry,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // load DATABASE_URL from a .env file if present

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env)");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = Arc::new(AppState {
        registry: Registry::new(pool).await,
    });

    let app = Router::new()
        .route("/api/auth/signup", post(auth::signup))
        .route("/api/auth/login", post(auth::login))
        .route("/api/servers", get(api::list_servers).post(api::create_server))
        .route("/api/servers/:server_id/channels", post(api::create_channel))
        .route("/api/channels/:channel_id/history", get(api::channel_history))
        .route("/ws/:channel_id", get(ws::ws_handler))
        .fallback_service(ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    println!("listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
