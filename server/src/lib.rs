pub(crate) mod api;
pub mod config;
mod health;
pub(crate) mod models;
pub(crate) mod senders;
pub(crate) mod sse;
mod state;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::state::{AppState, SharedState};

pub fn app(pool: PgPool) -> Router<SharedState> {
    let state = AppState::new(pool);
    Router::with_state(Arc::clone(&state))
        .route("/health", get(health::health))
        .nest("/api", api::app(Arc::clone(&state)))
        .nest("/sse", sse::app(Arc::clone(&state)))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_credentials(true)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
}

pub async fn pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
