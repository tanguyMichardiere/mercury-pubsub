use crate::constants::SESSION_COOKIE_NAME;
use axum::async_trait;
use axum::extract::{FromRequest, RequestParts, TypedHeader};
use axum::headers::Cookie;
use hyper::StatusCode;

pub struct SessionCookie {
    pub session_token: String,
}

#[async_trait]
impl<B> FromRequest<B> for SessionCookie
where
    B: axum::body::HttpBody + Send,
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match TypedHeader::<Cookie>::from_request(req).await {
            Ok(cookie_header) => match cookie_header.get(SESSION_COOKIE_NAME) {
                Some(session_cookie) => Ok(SessionCookie {
                    session_token: session_cookie.to_string(),
                }),
                // no session token in Cookie header
                None => Err(StatusCode::UNAUTHORIZED),
            },
            // no Cookie header
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
