use crate::models::channel::Channel;
use crate::senders::Senders;
use axum::extract::Path;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::{Extension, Json, Router};
use error::Result;
use futures::stream::Stream;
use serde_json::Value;
use sqlx::PgPool;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::{error, instrument};

pub fn app() -> Router {
    Router::new()
        .route("/:channel_name", get(subscribe).post(publish))
        .layer(Extension(Senders::new()))
}

#[instrument]
async fn subscribe(
    Path(channel_name): Path<String>,
    Extension(pool): Extension<PgPool>,
    Extension(mut senders): Extension<Senders>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>> {
    let channel = Channel::get(&pool, &channel_name).await?;
    let receiver = senders.get_receiver(&channel);
    Ok(
        Sse::new(BroadcastStream::new(receiver).filter_map(|result| {
            match result {
                Ok(value) => Some(Ok(Event::default()
                    .json_data(value)
                    .expect("invalid json from channel"))),
                Err(error) => {
                    error!(?error);
                    None
                }
            }
        }))
        .keep_alive(KeepAlive::default()),
    )
}

#[instrument]
async fn publish(
    Path(channel_name): Path<String>,
    Extension(pool): Extension<PgPool>,
    Extension(mut senders): Extension<Senders>,
    Json(body): Json<Value>,
) -> Result<String> {
    let channel = Channel::get(&pool, &channel_name).await?;
    channel.validate(&body)?;
    let sender = senders.get(&channel);
    Ok(format!("{}", sender.send(body).unwrap_or(0)))
}

mod error {
    use crate::models::channel;
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use jsonschema::ErrorIterator;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        ChannelError(#[from] channel::error::Error),
        // TODO: include information about the error
        #[error("Invalid data")]
        InvalidData,
    }

    impl<'a> From<ErrorIterator<'a>> for Error {
        fn from(_: ErrorIterator<'a>) -> Self {
            Self::InvalidData
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Error::ChannelError(error) => error.into_response(),
                Error::InvalidData => (StatusCode::UNPROCESSABLE_ENTITY, self).into_response(),
            }
        }
    }
}
