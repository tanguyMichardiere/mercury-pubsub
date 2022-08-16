use axum::body::HttpBody;
use axum::extract::{FromRequest, RequestParts, TypedHeader};
use axum::headers::{Cookie, HeaderValue};
use axum::{async_trait, BoxError};
use chrono::{DateTime, Utc};
use error::{Error, Result};
use sqlx::PgPool;

#[cfg(not(feature = "secure"))]
macro_rules! cookie_name {
    () => {
        "refreshToken"
    };
}
#[cfg(feature = "secure")]
macro_rules! cookie_name {
    () => {
        "__Host-refreshToken"
    };
}

macro_rules! cookie_attributes {
    () => {
        "SameSite=Strict; Secure; HttpOnly; Path=/"
    };
}

pub const LOGOUT_HEADER: &str = concat!(
    cookie_name!(),
    "=; expires=Thu, 01 Jan 1970 00:00:00 UTC; ",
    cookie_attributes!()
);

/// A refresh token, granted to the user when they signin, login or refresh their session.
#[derive(Debug)]
pub struct RefreshToken(String);

impl RefreshToken {
    /// Generate a new unique refresh token with the database.
    pub async fn new(pool: &PgPool) -> Result<Self> {
        Ok(Self(
            sqlx::query_scalar!(r#"SELECT encode(gen_random_bytes(48), 'base64')"#)
                .fetch_one(pool)
                .await
                .map(|option| option.expect("NULL from SELECT scalar"))?,
        ))
    }

    /// Get a (Set-Cookie) HeaderValue containing this refresh token.
    pub fn into_header_value(&self, expires: DateTime<Utc>) -> HeaderValue {
        format!(
            "{}={}; Expires={}; {}",
            cookie_name!(),
            self.0,
            expires.to_rfc2822(),
            cookie_attributes!()
        )
        .parse()
        .expect("wrong formatting")
    }
}

impl AsRef<str> for RefreshToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[async_trait]
impl<B> FromRequest<B> for RefreshToken
where
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self> {
        Ok(Self(
            TypedHeader::<Cookie>::from_request(req)
                .await?
                .get(cookie_name!())
                .ok_or(Error::MissingRefreshTokenCookie)?
                .to_owned(),
        ))
    }
}

pub mod error {
    use axum::extract::rejection::TypedHeaderRejection;
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use tracing::error;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        TypedHeaderRejection(#[from] TypedHeaderRejection),
        #[error("Missing refresh token in Cookie header")]
        MissingRefreshTokenCookie,
    }

    impl From<sqlx::Error> for Error {
        fn from(error: sqlx::Error) -> Self {
            error!(?error);
            panic!("unknown database error");
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Error::TypedHeaderRejection(typed_header_rejection) => {
                    typed_header_rejection.into_response()
                }
                Error::MissingRefreshTokenCookie => {
                    (StatusCode::UNAUTHORIZED, self).into_response()
                }
            }
        }
    }
}
