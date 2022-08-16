use crate::models::channel::Channel;
use crate::models::key::{Key, KeyType};
use crate::models::session::Session;
use axum::extract::Path;
use axum::routing::get;
use axum::{Extension, Json, Router};
use error::Result;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

pub fn app() -> Router {
    Router::new()
        .route("/", get(list_keys).post(create_key))
        .route(
            "/:key_id",
            get(list_channels).put(edit_channels).delete(delete_key),
        )
}

#[instrument]
async fn list_keys(Extension(pool): Extension<PgPool>, session: Session) -> Result<Json<Vec<Key>>> {
    Ok(Json(Key::get_all(&pool).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateKeyBody {
    r#type: KeyType,
}

#[instrument]
async fn create_key(
    Extension(pool): Extension<PgPool>,
    session: Session,
    Json(body): Json<CreateKeyBody>,
) -> Result<String> {
    let (key, secret) = Key::new(&pool, body.r#type).await?;
    Ok(format!("{};{}", key.get_id(), secret.as_ref()))
}

#[instrument]
async fn list_channels(
    Path(key_id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    session: Session,
) -> Result<Json<Vec<Channel>>> {
    let key = Key::get(&pool, key_id).await?;
    Ok(Json(Channel::get_from_key(&pool, &key).await?))
}

#[instrument]
async fn edit_channels(
    Path(key_id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    session: Session,
    Json(ids): Json<Vec<Uuid>>,
) -> Result<()> {
    let key = Key::get(&pool, key_id).await?;
    key.set_channels(&pool, ids).await?;
    Ok(())
}

#[instrument]
async fn delete_key(
    Path(key_id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    session: Session,
) -> Result<Json<Key>> {
    Ok(Json(Key::delete(&pool, key_id).await?))
}

mod error {
    use crate::models::{channel, key};
    use axum::response::IntoResponse;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        KeyError(#[from] key::error::Error),
        #[error(transparent)]
        ChannelError(#[from] channel::error::Error),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Error::KeyError(error) => error.into_response(),
                Error::ChannelError(error) => error.into_response(),
            }
        }
    }
}
