use crate::api::extract::validated_json::ValidatedJson;
use crate::models::channel::Channel;
use crate::models::session::Session;
use axum::routing::get;
use axum::{Extension, Json, Router};
use error::Result;
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use tracing::instrument;
use validator::Validate;

pub fn app() -> Router {
    Router::new().route("/", get(list_channels).post(create_channel))
}

#[instrument]
async fn list_channels(
    Extension(pool): Extension<PgPool>,
    session: Session,
) -> Result<Json<Vec<Channel>>> {
    Ok(Json(Channel::get_all(&pool).await?))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct CreateChannelBody {
    #[validate(length(min = 4, max = 16))]
    name: String,
    schema: Value,
}

#[instrument]
async fn create_channel(
    Extension(pool): Extension<PgPool>,
    session: Session,
    ValidatedJson(body): ValidatedJson<CreateChannelBody>,
) -> Result<Json<Channel>> {
    Ok(Json(Channel::new(&pool, &body.name, &body.schema).await?))
}

mod error {
    use crate::models::channel;
    use axum::response::IntoResponse;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        ChannelError(#[from] channel::error::Error),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Error::ChannelError(error) => error.into_response(),
            }
        }
    }
}
