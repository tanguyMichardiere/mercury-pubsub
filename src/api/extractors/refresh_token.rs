use axum::async_trait;
use axum::extract::{FromRequest, RequestParts, TypedHeader};
use axum::headers::Cookie;
use hyper::StatusCode;
use std::ops::Deref;

#[cfg(not(feature = "secure"))]
pub const REFRESH_TOKEN_COOKIE_NAME: &'static str = "refreshToken";
#[cfg(feature = "secure")]
pub const REFRESH_TOKEN_COOKIE_NAME: &'static str = "__Host-refreshToken";

/// A refresh token, granted to the user when they signin, login or refresh their session.
#[derive(Debug)]
pub struct RefreshToken(String);

impl Deref for RefreshToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<B> FromRequest<B> for RefreshToken
where
    B: axum::body::HttpBody + Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(Self(
            TypedHeader::<Cookie>::from_request(req)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing Cookie header"))?
                .get(REFRESH_TOKEN_COOKIE_NAME)
                .ok_or((
                    StatusCode::UNAUTHORIZED,
                    "Missing refresh token in Cookie header",
                ))?
                .to_owned(),
        ))
    }
}
