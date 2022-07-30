pub mod auth;
pub mod error_handlers;
pub mod extractors;

use axum::{Extension, Router};
use sqlx::PgPool;

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .nest("/auth", auth::app())
        .layer(Extension(pool))
}
