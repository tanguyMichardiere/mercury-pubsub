use axum::body::HttpBody;
use axum::extract::FromRequest;
use axum::{async_trait, BoxError, Json};
use hyper::Request;
use serde::de::DeserializeOwned;
use validator::Validate;

use self::error::{Error, Result};

pub(crate) struct ValidatedJson<T>(pub(crate) T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(Self(value))
    }
}

mod error {
    use axum::extract::rejection::JsonRejection;
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use tracing::debug;
    use validator::ValidationErrors;

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum Error {
        #[error(transparent)]
        JsonRejection(#[from] JsonRejection),
        #[error(transparent)]
        ValidationErrors(#[from] ValidationErrors),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            debug!(?self);
            match self {
                Self::JsonRejection(json_rejection) => json_rejection.into_response(),
                Self::ValidationErrors(validation_errors) => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    validation_errors.to_string(),
                )
                    .into_response(),
            }
        }
    }
}
