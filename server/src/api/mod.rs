pub(crate) mod channels;
pub(crate) mod extract;
pub(crate) mod keys;
pub(crate) mod users;

use axum::Router;

pub(crate) fn app() -> Router {
    Router::new()
        .nest("/users", users::app())
        .nest("/channels", channels::app())
        .nest("/keys", keys::app())
}
