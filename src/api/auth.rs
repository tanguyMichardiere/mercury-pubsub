use crate::api::extract::validated_json::ValidatedJson;
use crate::models::refresh_token::{RefreshToken, LOGOUT_HEADER};
use crate::models::session::Session;
use crate::models::user::User;
use axum::headers::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use error::Result;
use hyper::StatusCode;
use hyper::{header, HeaderMap};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use validator::Validate;

pub fn app() -> Router {
    Router::new()
        .route("/session", get(session))
        .route("/login", post(login))
        .route("/refresh", post(refresh_session))
        .route("/logout", post(logout))
        .route("/create-user", post(create_user))
        .route("/user-count", get(user_count))
}

/// Get the session object for a logged in user
#[instrument]
async fn session(Extension(pool): Extension<PgPool>, session: Session) -> Json<Session> {
    Json(session)
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    #[validate(length(min = 4, max = 16))]
    name: String,
    #[validate(length(min = 8))]
    password: String,
}

/// Login with a user name and password
#[instrument]
async fn login(
    Extension(pool): Extension<PgPool>,
    ValidatedJson(body): ValidatedJson<LoginBody>,
) -> Result<(HeaderMap, Json<Session>)> {
    let (session, refresh_token) = Session::new(
        &pool,
        User::get_by_name_and_password(&pool, &body.name, &body.password).await?,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        refresh_token.into_header_value(session.expires),
    );
    Ok((headers, Json(session)))
}

/// Refresh the authenticated user's session, and get back the session object and authentication
/// cookie
#[instrument]
async fn refresh_session(
    Extension(pool): Extension<PgPool>,
    refresh_token: RefreshToken,
) -> Result<(HeaderMap, Json<Session>)> {
    let session = Session::refresh(&pool, &refresh_token).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        refresh_token.into_header_value(session.expires),
    );
    Ok((headers, Json(session)))
}

/// Log the user out.
#[instrument]
async fn logout(Extension(pool): Extension<PgPool>, refresh_token: RefreshToken) -> HeaderMap {
    // don't interrupt in case the session couldn't be deleted
    Session::delete(&pool, &refresh_token).await.ok();
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, HeaderValue::from_static(LOGOUT_HEADER));
    headers
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct CreateUserBody {
    #[validate(length(min = 4, max = 16))]
    name: String,
    #[validate(length(min = 8))]
    password: String,
}

/// Create a user.
///
/// If there are no users yet, this doesn't require authentication and logs the new user in.
#[instrument]
async fn create_user(
    Extension(pool): Extension<PgPool>,
    session: Option<Session>,
    ValidatedJson(body): ValidatedJson<CreateUserBody>,
) -> Result<Response> {
    Ok(if User::count(&pool).await? == 0 {
        // no user yet
        let (session, refresh_token) =
            Session::new(&pool, User::new(&pool, &body.name, &body.password).await?).await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SET_COOKIE,
            refresh_token.into_header_value(session.expires),
        );
        (headers, Json(session)).into_response()
    } else {
        // there are users already, only allow authenticated users to create more
        if session.is_some() {
            User::new(&pool, &body.name, &body.password).await?;
            (StatusCode::CREATED, "New user created successfully").into_response()
        } else {
            (
                StatusCode::UNAUTHORIZED,
                "Must be authenticated to create a user",
            )
                .into_response()
        }
    })
}

/// Get the number of existing users.
#[instrument]
async fn user_count(Extension(pool): Extension<PgPool>) -> Result<String> {
    Ok(User::count(&pool).await.map(|count| format!("{}", count))?)
}

mod error {
    use crate::models::{session, user};
    use axum::response::IntoResponse;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        SessionError(#[from] session::error::Error),
        #[error(transparent)]
        UserError(#[from] user::error::Error),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Error::SessionError(error) => error.into_response(),
                Error::UserError(error) => error.into_response(),
            }
        }
    }
}
