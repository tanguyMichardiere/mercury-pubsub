pub mod auth;
pub mod channels;
pub mod extract;
pub mod keys;

use axum::Router;

pub fn app() -> Router {
    Router::new()
        .nest("/auth", auth::app())
        .nest("/channels", channels::app())
        .nest("/keys", keys::app())
}
