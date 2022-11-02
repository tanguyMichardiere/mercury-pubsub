pub(crate) mod api;
pub mod config;
pub mod database;
mod health;
pub(crate) mod models;
pub(crate) mod senders;
pub(crate) mod sse;
mod state;

use std::sync::Arc;

use anyhow::Result;
use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::state::{AppState, SharedState};

pub async fn app() -> Result<Router<SharedState>> {
    let state = AppState::new().await?;
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let trace_layer = TraceLayer::new_for_http();

    let router = Router::with_state(Arc::clone(&state))
        .route("/health", get(health::health))
        .nest("/api", api::app(Arc::clone(&state)))
        .nest("/sse", sse::app(Arc::clone(&state)))
        .layer(cors_layer)
        .layer(trace_layer);

    Ok(router)
}
