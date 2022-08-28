use std::convert::Infallible;

use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::{Json, Router};
use futures::stream::Stream;
use serde_json::Value;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::{error, instrument};

use crate::models::channel::Channel;
use crate::models::key::Key;
use crate::state::SharedState;

use error::{Error, Result};

pub(crate) fn app(state: SharedState) -> Router<SharedState> {
    Router::with_state(state).route("/:channel_name", get(subscribe).post(publish))
}

#[instrument]
async fn subscribe(
    State(state): State<SharedState>,
    key: Key,
    Path(channel_name): Path<String>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>> {
    let channel = Channel::get_by_name(&state.read().await.pool, &channel_name).await?;
    if key.is_subscriber() && key.authorizes(&state.read().await.pool, &channel).await? {
        let receiver = state.write().await.senders.get_receiver(&channel);
        let stream = BroadcastStream::new(receiver).filter_map(|result| match result {
            Ok(value) => Some(Ok(Event::default()
                .json_data(value)
                .expect("invalid JSON from channel"))),
            Err(error) => {
                error!(?error);
                None
            }
        });
        Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
    } else {
        Err(Error::UnauthorizedChannel)
    }
}

#[instrument]
async fn publish(
    State(state): State<SharedState>,
    key: Key,
    Path(channel_name): Path<String>,
    Json(body): Json<Value>,
) -> Result<String> {
    let channel = Channel::get_by_name(&state.read().await.pool, &channel_name).await?;
    if key.is_publisher() && key.authorizes(&state.read().await.pool, &channel).await? {
        if channel.is_valid(&body) {
            let sender = state.write().await.senders.get(&channel);
            Ok(format!("{}", sender.send(body).unwrap_or(0)))
        } else {
            Err(Error::from(
                channel
                    .validate(&body)
                    .expect_err("instance passes validate but not is_valid"),
            ))
        }
    } else {
        Err(Error::UnauthorizedChannel)
    }
}

mod error {
    use axum::response::IntoResponse;
    use axum::Json;
    use hyper::StatusCode;
    use jsonschema::ErrorIterator;
    use serde::Serialize;
    use serde_json::Value;
    use tracing::debug;

    use crate::models::{channel, key};

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct ValidationError {
        kind: String,
        instance: Value,
        instance_path: String,
        schema_path: String,
    }

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum Error {
        #[error(transparent)]
        ChannelError(#[from] channel::error::Error),
        #[error(transparent)]
        KeyError(#[from] key::error::Error),
        #[error("Invalid data")]
        InvalidData(Vec<ValidationError>),
        #[error("Unauthorized channel")]
        UnauthorizedChannel,
    }

    impl<'a> From<ErrorIterator<'a>> for Error {
        fn from(error_iterator: ErrorIterator<'a>) -> Self {
            Self::InvalidData(
                error_iterator
                    .map(|error| ValidationError {
                        kind: format!("{:?}", error.kind),
                        instance: error.instance.into_owned(),
                        instance_path: error.instance_path.to_string(),
                        schema_path: error.schema_path.to_string(),
                    })
                    .collect(),
            )
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            debug!(?self);
            match self {
                Error::ChannelError(error) => error.into_response(),
                Error::KeyError(error) => error.into_response(),
                Error::InvalidData(errors) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, Json(errors)).into_response()
                }
                Error::UnauthorizedChannel => {
                    (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
                }
            }
        }
    }
}
