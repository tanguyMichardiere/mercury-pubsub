use axum::http::Request;
use axum::response::Response;
use axum::Router;
use chrono::DateTime;
use hyper::{header, Body, StatusCode};
use serde_json::{from_str, json, to_string, Value};
use server::app;
use sqlx::PgPool;
use std::str;
use tower::{Service, ServiceExt};

const REFRESH_TOKEN_COOKIE_NAME: &'static str = "refreshToken";

fn get_refresh_token(response: &Response) -> &str {
    &response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find(|header_value| {
            // find the header with the right cookie name
            header_value
                .to_str()
                .unwrap()
                .starts_with(REFRESH_TOKEN_COOKIE_NAME)
        })
        .unwrap()
        .to_str()
        // extract the cookie value (its length is always 64)
        .unwrap()[(REFRESH_TOKEN_COOKIE_NAME.len() + 1)..(REFRESH_TOKEN_COOKIE_NAME.len() + 1 + 64)]
}

async fn get_body(response: Response) -> Value {
    from_str::<Value>(
        str::from_utf8(&hyper::body::to_bytes(response.into_body()).await.unwrap()).unwrap(),
    )
    .unwrap()
}

async fn create_user(app: &mut Router, name: &str, password: &str) -> Response {
    app.ready()
        .await
        .unwrap()
        .call(
            Request::post("/api/auth/create-user")
                .header(header::CONTENT_TYPE, "application/json")
                .body(
                    to_string(&json!({"name": name, "password": password}))
                        .unwrap()
                        .into(),
                )
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn get_session(app: &mut Router, access_token: &str) -> Response {
    app.ready()
        .await
        .unwrap()
        .call(
            Request::get("/api/auth/session")
                .header("Authorization", format!("Bearer {access_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn refresh_session(app: &mut Router, refresh_token: &str) -> Response {
    app.ready()
        .await
        .unwrap()
        .call(
            Request::post("/api/auth/refresh")
                .header(
                    "Cookie",
                    format!("{REFRESH_TOKEN_COOKIE_NAME}={refresh_token}"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[sqlx::test]
async fn first_signin(pool: PgPool) {
    let mut app = app(pool);
    let response = create_user(&mut app, "admin", "password").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = get_body(response).await;
    let access_token = body["accessToken"].as_str().unwrap();
    assert_eq!(access_token.len(), 64);
}

#[sqlx::test]
async fn signin_then_get_session(pool: PgPool) {
    let mut app = app(pool);
    let signin_response = create_user(&mut app, "admin", "password").await;
    assert_eq!(signin_response.status(), StatusCode::OK);
    let signin_body = get_body(signin_response).await;
    let access_token = signin_body["accessToken"].as_str().unwrap();

    let get_session_response = get_session(&mut app, access_token).await;
    assert_eq!(get_session_response.status(), StatusCode::OK);
    let get_session_body = get_body(get_session_response).await;

    assert!(get_session_body["accessToken"].is_null());
    assert_eq!(get_session_body["expires"], signin_body["expires"]);
    assert_eq!(get_session_body["user"], signin_body["user"]);
}

#[sqlx::test]
async fn signin_then_refresh_session(pool: PgPool) {
    let mut app = app(pool);
    let signin_response = create_user(&mut app, "admin", "password").await;
    assert_eq!(signin_response.status(), StatusCode::OK);
    let refresh_token = get_refresh_token(&signin_response).to_owned();
    let signin_body = get_body(signin_response).await;

    let refresh_session_response = refresh_session(&mut app, &refresh_token).await;
    assert_eq!(refresh_session_response.status(), StatusCode::OK);
    let refresh_session_body = get_body(refresh_session_response).await;

    assert_ne!(
        refresh_session_body["accessToken"],
        signin_body["accessToken"]
    );
    assert!(
        DateTime::parse_from_rfc3339(signin_body["expires"].as_str().unwrap()).unwrap()
            < DateTime::parse_from_rfc3339(refresh_session_body["expires"].as_str().unwrap())
                .unwrap()
    );
    assert_eq!(refresh_session_body["user"], signin_body["user"]);
}
