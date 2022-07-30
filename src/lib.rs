pub mod api;
pub mod config;
pub mod models;

use axum::routing::{get_service, MethodRouter};
use axum::Router;
use hyper::StatusCode;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::error;

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .nest("/api", api::app(pool))
        .fallback(static_dir_service())
        .layer(TraceLayer::new_for_http())
}

pub async fn pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().connect(&database_url).await
}

pub fn static_dir_service() -> MethodRouter {
    get_service(ServeDir::new("static").fallback(ServeFile::new("static/404.html")))
        .handle_error(handle_io_error)
}

async fn handle_io_error(error: io::Error) -> StatusCode {
    error!(?error);
    StatusCode::INTERNAL_SERVER_ERROR
}
