use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRequest, RequestParts};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::{async_trait, TypedHeader};
use error::Result;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub struct AccessToken(String);

impl AccessToken {
    pub async fn new(pool: &PgPool) -> Result<Self> {
        Ok(Self(
            sqlx::query_scalar!(r#"SELECT encode(gen_random_bytes(48), 'base64')"#)
                .fetch_one(pool)
                .await
                .map(|option| option.expect("NULL from SELECT scalar"))?,
        ))
    }
}

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[async_trait]
impl<B> FromRequest<B> for AccessToken
where
    B: axum::body::HttpBody + Send,
{
    type Rejection = TypedHeaderRejection;

    async fn from_request(req: &mut RequestParts<B>) -> std::result::Result<Self, Self::Rejection> {
        Ok(Self(
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await?
                .token()
                .to_owned(),
        ))
    }
}

pub mod error {
    use axum::response::IntoResponse;
    use tracing::error;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {}

    impl From<sqlx::Error> for Error {
        fn from(error: sqlx::Error) -> Self {
            error!(?error);
            panic!("unknown database error");
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {}
        }
    }
}
