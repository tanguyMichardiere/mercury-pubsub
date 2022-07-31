use crate::api::error_handlers::handle_database_error;
use crate::api::extractors::refresh_token::{RefreshToken, REFRESH_TOKEN_COOKIE_NAME};
use crate::models::session::Session;
use crate::models::user::User;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use hyper::StatusCode;
use hyper::{header, HeaderMap};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use tracing::{debug, instrument};
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
async fn session(
    Extension(pool): Extension<PgPool>,
    session: Option<Session>,
) -> Result<Session, Json<Value>> {
    session.ok_or(Json(Value::Null))
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
    Json(body): Json<LoginBody>,
) -> Result<Session, (StatusCode, &'static str)> {
    match User::get_by_name_and_password(&pool, &body.name, &body.password)
        .await
        .map_err(handle_database_error)?
    {
        Some(user) => Session::new(&pool, user)
            .await
            .map_err(handle_database_error),
        None => {
            debug!("no user found with this name and password");
            Err((
                StatusCode::NOT_FOUND,
                "No user found with this name and password",
            ))
        }
    }
}

/// Refresh the authenticated user's session, and get back the session object and authentication
/// cookie
#[instrument]
async fn refresh_session(
    Extension(pool): Extension<PgPool>,
    refresh_token: RefreshToken,
) -> Result<Session, (StatusCode, &'static str)> {
    Session::refresh(&pool, &refresh_token)
        .await
        .map_err(handle_database_error)?
        .ok_or((
            StatusCode::NOT_FOUND,
            "No session found with this refresh token",
        ))
}

/// Log the user out.
#[instrument]
async fn logout(Extension(pool): Extension<PgPool>, refresh_token: RefreshToken) -> HeaderMap {
    // don't interrupt in case the session could't be deleted
    Session::delete(&pool, &refresh_token).await.ok();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        format!(
            "{REFRESH_TOKEN_COOKIE_NAME}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; SameSite=Strict; Secure; HttpOnly; Path=/",
        )
        .parse()
        .expect("wrong formatting"),
    );
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
    Json(body): Json<CreateUserBody>,
) -> Result<Session, (StatusCode, &'static str)> {
    if User::count(&pool).await.map_err(handle_database_error)? == 0 {
        // no user yet
        Session::new(
            &pool,
            User::new(&pool, &body.name, &body.password)
                .await
                .map_err(handle_database_error)?,
        )
        .await
        .map_err(handle_database_error)
    } else {
        // there are users already, only allow authenticated users to create more
        if session.is_some() {
            User::new(&pool, &body.name, &body.password)
                .await
                .map_err(handle_database_error)?;
            Err((StatusCode::CREATED, "New user created successfully"))
        } else {
            debug!("must be authenticated to create a user");
            Err((
                StatusCode::UNAUTHORIZED,
                "Must be authenticated to create a user",
            ))
        }
    }
}

/// Get the number of existing users.
#[instrument]
async fn user_count(
    Extension(pool): Extension<PgPool>,
) -> Result<String, (StatusCode, &'static str)> {
    User::count(&pool)
        .await
        .map(|count| format!("{}", count))
        .map_err(handle_database_error)
}
