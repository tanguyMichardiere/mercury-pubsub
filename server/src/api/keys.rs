use axum::extract::Path;
use axum::routing::get;
use axum::{Extension, Json, Router};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::models::channel::Channel;
use crate::models::key::{Key, KeyType};
use crate::models::user::User;

use error::Result;

pub(crate) fn app() -> Router {
    Router::new()
        .route("/", get(list_keys).post(create_key))
        .route(
            "/:id",
            get(list_channels).patch(edit_channels).delete(delete_key),
        )
}

/// Get all keys.
#[instrument]
async fn list_keys(Extension(pool): Extension<PgPool>, user: User) -> Result<Json<Vec<Key>>> {
    Ok(Json(Key::get_all(&pool).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateKeyBody {
    r#type: KeyType,
}

/// Create a key.
#[instrument]
async fn create_key(
    Extension(pool): Extension<PgPool>,
    user: User,
    Json(body): Json<CreateKeyBody>,
) -> Result<String> {
    let (key, secret) = Key::new(&pool, body.r#type).await?;
    Ok(format!("{};{}", key.id, secret.as_ref()))
}

/// Get all channels that a key authorizes.
#[instrument]
async fn list_channels(
    Extension(pool): Extension<PgPool>,
    user: User,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Channel>>> {
    let key = Key::get(&pool, id).await?;
    Ok(Json(Channel::get_from_key(&pool, &key).await?))
}

/// Set the channels that a key authorizes.
#[instrument]
async fn edit_channels(
    Extension(pool): Extension<PgPool>,
    user: User,
    Path(id): Path<Uuid>,
    Json(body): Json<Vec<Uuid>>,
) -> Result<()> {
    let key = Key::get(&pool, id).await?;
    key.set_channels(&pool, body).await?;
    Ok(())
}

/// Delete a key.
#[instrument]
async fn delete_key(
    Extension(pool): Extension<PgPool>,
    user: User,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let key = Key::get(&pool, id).await?;
    key.delete(&pool).await?;
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
