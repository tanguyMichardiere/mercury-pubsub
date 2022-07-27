use axum::routing::{get_service, MethodRouter};
use axum::{response::IntoResponse, Extension, Router};
use hyper::StatusCode;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::io;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::error;

pub mod config;
pub mod hello;

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .nest("/api", api_app().layer(Extension(pool)))
        .fallback(static_dir_service())
        .layer(TraceLayer::new_for_http())
}

fn api_app() -> Router {
    Router::new().nest("/hello", hello::app())
}

pub async fn pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().connect(&database_url).await
}

pub fn static_dir_service() -> MethodRouter {
    get_service(ServeDir::new("static").fallback(ServeFile::new("static/404.html")))
        .handle_error(handle_io_error)
}

async fn handle_io_error(error: io::Error) -> impl IntoResponse {
    error!(?error);
    StatusCode::INTERNAL_SERVER_ERROR
}
