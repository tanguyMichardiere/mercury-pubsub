pub(crate) mod channels;
pub(crate) mod extract;
pub(crate) mod keys;
pub(crate) mod users;

use std::sync::Arc;

use axum::Router;

use crate::state::SharedState;

pub(crate) fn app(state: SharedState) -> Router<SharedState> {
    Router::with_state(Arc::clone(&state))
        .nest("/users", users::app(Arc::clone(&state)))
        .nest("/channels", channels::app(Arc::clone(&state)))
        .nest("/keys", keys::app(Arc::clone(&state)))
}
