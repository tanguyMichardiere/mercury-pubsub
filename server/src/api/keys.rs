use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use hyper::StatusCode;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::models::channel::Channel;
use crate::models::key::{Key, KeyType};
use crate::models::user::User;
use crate::state::SharedState;

use error::Result;

pub(crate) fn app(state: SharedState) -> Router<SharedState> {
    Router::with_state(state)
        .route("/", get(list_keys).post(create_key))
        .route("/:id", get(list_channels).delete(delete_key))
}

/// Get all keys.
#[instrument]
async fn list_keys(State(state): State<SharedState>, user: User) -> Result<Json<Vec<Key>>> {
    Ok(Json(Key::get_all(&state.read().await.pool).await?))
}

#[derive(Debug, Deserialize)]
struct CreateKeyBody {
    r#type: KeyType,
    channels: Vec<Uuid>,
}

/// Create a key.
#[instrument]
async fn create_key(
    State(state): State<SharedState>,
    user: User,
    Json(body): Json<CreateKeyBody>,
) -> Result<String> {
    // TODO: next 2 instructions in 1 method
    let (key, secret) = Key::new(&state.read().await.pool, body.r#type, body.channels).await?;
    Ok(format!("{};{}", key.id, secret.as_ref()))
}

/// Get all channels that a key authorizes.
#[instrument]
async fn list_channels(
    State(state): State<SharedState>,
    user: User,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Channel>>> {
    let key = Key::get(&state.read().await.pool, id).await?;
    Ok(Json(
        Channel::get_from_key(&state.read().await.pool, &key).await?,
    ))
}

/// Delete a key.
#[instrument]
async fn delete_key(
    State(state): State<SharedState>,
    user: User,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let key = Key::get(&state.read().await.pool, id).await?;
    key.delete(&state.read().await.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

mod error {
    use axum::response::IntoResponse;
    use tracing::debug;

    use crate::models::{channel, key};

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum Error {
        #[error(transparent)]
        KeyError(#[from] key::error::Error),
        #[error(transparent)]
        ChannelError(#[from] channel::error::Error),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            debug!(?self);
            match self {
                Error::KeyError(error) => error.into_response(),
                Error::ChannelError(error) => error.into_response(),
            }
        }
    }
}
