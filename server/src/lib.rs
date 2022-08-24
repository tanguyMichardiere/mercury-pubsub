pub mod config;

pub(crate) mod api;
pub(crate) mod models;
pub(crate) mod senders;
pub(crate) mod sse;

use axum::{Extension, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .nest("/api", api::app())
        .nest("/sse", sse::app())
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http())
}

pub async fn pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
