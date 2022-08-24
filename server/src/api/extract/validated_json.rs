use axum::body::HttpBody;
use axum::extract::{FromRequest, RequestParts};
use axum::BoxError;
use axum::{async_trait, Json};
use serde::de::DeserializeOwned;
use validator::Validate;

use error::Error;

pub(crate) struct ValidatedJson<T>(pub(crate) T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req).await?;
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
