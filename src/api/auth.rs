use crate::api::extractors::session_cookie::SessionCookie;
use crate::models::session::{Session, SessionWithUser};
use crate::models::user::User;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

pub fn app() -> Router {
    Router::new()
        .route("/", get(session).post(login))
        .route("/create-user", post(create_user))
}

/// Get the session object and refresh the authentication cookie
async fn session(
    Extension(pool): Extension<PgPool>,
    session_cookie: Option<SessionCookie>,
) -> Result<SessionWithUser, StatusCode> {
    match session_cookie {
        Some(session_cookie) => match Session::get(&pool, &session_cookie.session_token).await {
            Ok(mut session) => {
                session.extend(&pool).await.unwrap();
                Ok(session.with_user(&pool).await.unwrap())
            }
            Err(_) => Err(StatusCode::NO_CONTENT),
        },
        None => Err(StatusCode::NO_CONTENT),
    }
}

#[derive(Deserialize)]
struct LoginBody {
    name: String,
    password: String,
}

/// Login with a user name and password, and get the session object and authentication cookie
async fn login(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<LoginBody>,
) -> Result<SessionWithUser, StatusCode> {
    match User::get_by_name_and_password(&pool, &body.name, &body.password).await {
        Ok(user) => Ok(SessionWithUser::new(&pool, user).await.unwrap()),
        Err(_) => todo!(),
    }
}

#[derive(Deserialize)]
struct SigninBody {
    name: String,
    password: String,
}

/// Create a user
///
/// If there are no users yet, this doesn't require authentication and returns the new user's
/// session
async fn create_user(
    Extension(pool): Extension<PgPool>,
    session_cookie: Option<SessionCookie>,
    Json(body): Json<SigninBody>,
) -> Result<SessionWithUser, StatusCode> {
    match User::count(&pool).await.unwrap() {
        0 => Ok(SessionWithUser::new(
            &pool,
            User::new(&pool, &body.name, &body.password).await.unwrap(),
        )
        .await
        .unwrap()),
        _ => match session_cookie {
            Some(_) => {
                User::new(&pool, &body.name, &body.password).await.unwrap();
                Err(StatusCode::CREATED)
            }
            None => Err(StatusCode::UNAUTHORIZED),
        },
    }
}
