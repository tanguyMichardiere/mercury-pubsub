use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use hyper::StatusCode;
use jsonschema::JSONSchema;
use serde::Deserialize;
use serde_json::Value;
use tracing::instrument;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use self::error::Result;
use crate::api::extract::validated_json::ValidatedJson;
use crate::models::channel::Channel;
use crate::models::user::User;
use crate::state::SharedState;

pub(crate) fn app(state: SharedState) -> Router<SharedState> {
    Router::with_state(state)
        .route("/", get(list_channels).post(create_channel))
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
async fn list_channels(State(state): State<SharedState>, user: User) -> Result<Json<Vec<Channel>>> {
    Ok(Json(Channel::get_all(&state.read().await.pool).await?))
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
    State(state): State<SharedState>,
    user: User,
    ValidatedJson(body): ValidatedJson<CreateChannelBody>,
) -> Result<Json<Channel>> {
    Ok(Json(
        Channel::new(&state.read().await.pool, &body.name, &body.schema).await?,
    ))
}

/// Delete a channel.
#[instrument]
async fn delete_channel(
    State(state): State<SharedState>,
    user: User,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let channel = Channel::get(&state.read().await.pool, id).await?;
    channel.delete(&state.read().await.pool).await?;
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
