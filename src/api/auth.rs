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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    name: String,
    password: String,
}

/// Login with a user name and password
#[instrument]
async fn login(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<LoginBody>,
) -> Result<Session, StatusCode> {
    match User::get_by_name_and_password(&pool, &body.name, &body.password)
        .await
        .map_err(handle_database_error)?
    {
        Some(user) => Session::new(&pool, user)
            .await
            .map_err(handle_database_error),
        None => {
            debug!("no user found with this name and password");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Refresh the authenticated user's session, and get back the session object and authentication
/// cookie
#[instrument]
async fn refresh_session(
    Extension(pool): Extension<PgPool>,
    refresh_token: RefreshToken,
) -> Result<Session, StatusCode> {
    Session::refresh(&pool, &refresh_token)
        .await
        .map_err(handle_database_error)?
        .ok_or(StatusCode::NOT_FOUND)
}

/// Log the user out
async fn logout() -> HeaderMap {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateUserBody {
    name: String,
    password: String,
}

/// Create a user
///
/// If there are no users yet, this doesn't require authentication and logs the new user in
#[instrument]
async fn create_user(
    Extension(pool): Extension<PgPool>,
    session: Option<Session>,
    Json(body): Json<CreateUserBody>,
) -> Result<Session, StatusCode> {
    match User::count(&pool).await.map_err(handle_database_error)? {
        // no user yet
        0 => {
            let user = User::new(&pool, &body.name, &body.password)
                .await
                .map_err(handle_database_error)?;
            Session::new(&pool, user)
                .await
                .map_err(handle_database_error)
        }
        // there already exist users, only allow logged users to create more
        _ => {
            if session.is_some() {
                User::new(&pool, &body.name, &body.password)
                    .await
                    .map_err(handle_database_error)?;
                Err(StatusCode::CREATED)
            } else {
                debug!("must be authenticated to create a user");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

/// Get the number of existing users.
#[instrument]
async fn user_count(Extension(pool): Extension<PgPool>) -> Result<String, StatusCode> {
    User::count(&pool)
        .await
        .map(|count| format!("{}", count))
        .map_err(handle_database_error)
}
