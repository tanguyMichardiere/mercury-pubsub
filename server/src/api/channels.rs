use axum::extract::Path;
use axum::routing::{delete, get, patch};
use axum::{Extension, Json, Router};
use hyper::StatusCode;
use jsonschema::JSONSchema;
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::api::extract::validated_json::ValidatedJson;
use crate::models::channel::Channel;
use crate::models::user::User;

use error::Result;

pub(crate) fn app() -> Router {
    Router::new()
        .route("/", get(list_channels).post(create_channel))
        .route("/rename/:id", patch(rename_channel))
        .route("/change-schema/:id", patch(change_schema))
        .route("/:id", delete(delete_channel))
}

/// Validate the JSON schema (for the validator crate).
fn validate_schema(schema: &Value) -> std::result::Result<(), ValidationError> {
    JSONSchema::compile(schema)
        .map(|_| ())
        // TODO: add error details
        .map_err(|_| ValidationError::new("Invalid schema"))
}

/// Get all channels.
#[instrument]
async fn list_channels(
    Extension(pool): Extension<PgPool>,
    user: User,
) -> Result<Json<Vec<Channel>>> {
    Ok(Json(Channel::get_all(&pool).await?))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateChannelBody {
    #[validate(length(min = 4, max = 16))]
    name: String,
    #[validate(custom = "validate_schema")]
    schema: Value,
}

/// Create a channel.
#[instrument]
async fn create_channel(
    Extension(pool): Extension<PgPool>,
    user: User,
    ValidatedJson(body): ValidatedJson<CreateChannelBody>,
) -> Result<Json<Channel>> {
    Ok(Json(Channel::new(&pool, &body.name, &body.schema).await?))
}

#[derive(Debug, Deserialize, Validate)]
struct RenameChannelBody {
    #[validate(length(min = 4, max = 16))]
    name: String,
}

/// Rename a channel.
#[instrument]
async fn rename_channel(
    Extension(pool): Extension<PgPool>,
    user: User,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<RenameChannelBody>,
) -> Result<Json<Channel>> {
    let mut channel = Channel::get(&pool, id).await?;
    channel.rename(&pool, &body.name).await?;
    Ok(Json(channel))
}

#[derive(Debug, Deserialize, Validate)]
struct ChangeSchemaBody {
    #[validate(custom = "validate_schema")]
    schema: Value,
}

/// Change a channel's schema.
#[instrument]
async fn change_schema(
    Extension(pool): Extension<PgPool>,
    user: User,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<ChangeSchemaBody>,
) -> Result<Json<Channel>> {
    let mut channel = Channel::get(&pool, id).await?;
    channel.change_schema(&pool, &body.schema).await?;
    Ok(Json(channel))
}

/// Delete a channel.
#[instrument]
async fn delete_channel(
    Extension(pool): Extension<PgPool>,
    user: User,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let channel = Channel::get(&pool, id).await?;
    channel.delete(&pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

mod error {
    use axum::response::IntoResponse;
    use tracing::debug;

    use crate::models::channel;

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum Error {
        #[error(transparent)]
        ChannelError(#[from] channel::error::Error),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            debug!(?self);
            match self {
                Error::ChannelError(error) => error.into_response(),
            }
        }
    }
}
