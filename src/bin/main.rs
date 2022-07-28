use dotenv::dotenv;
use server::config::{config, LogFormat};
use server::{app, pool};
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing_subscriber::EnvFilter;

fn tracing_init(env_filter: EnvFilter, log_format: LogFormat) {
    let subscriber_builder = tracing_subscriber::fmt().with_env_filter(env_filter);
    match log_format {
        LogFormat::Json => subscriber_builder.json().init(),
        LogFormat::Pretty => subscriber_builder.pretty().init(),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = config().expect("config parsing");

    tracing_init(EnvFilter::new(config.log), config.log_format);

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable");
    let pool = pool(&database_url).await.expect("database connection");
    let app = app(pool);

    axum::Server::bind(&SocketAddr::new(
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        config.port,
    ))
    .serve(app.into_make_service())
    .await
    .expect("server launch");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use axum::response::Response;
    use serde_json::{from_str, json, to_string, Value};
    use server::constants::SESSION_COOKIE_NAME;
    use std::str;
    use tower::ServiceExt;

    fn get_session_token(response: &Response) -> &str {
        &response
            .headers()
            .get_all("Set-Cookie")
            .iter()
            .find(|header_value| {
                // find the header with the right cookie name
                header_value
                    .to_str()
                    .unwrap()
                    .starts_with(SESSION_COOKIE_NAME)
            })
            .unwrap()
            .to_str()
            // extract the cookie value (its length is always 64)
            .unwrap()[(SESSION_COOKIE_NAME.len() + 1)..(SESSION_COOKIE_NAME.len() + 1 + 64)]
    }

    #[sqlx_database_tester::test(pool(variable = "pool"))]
    async fn first_signin() {
        let app = app(pool);

        let response = app
            .oneshot(
                Request::post("/api/auth/create-user")
                    .header("Content-Type", "application/json")
                    .body(
                        to_string(&json!({"name": "name", "password": "password"}))
                            .unwrap()
                            .into(),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let session_token = get_session_token(&response);

        assert_eq!(session_token.len(), 64);

        from_str::<Value>(
            str::from_utf8(&hyper::body::to_bytes(response.into_body()).await.unwrap()).unwrap(),
        )
        .unwrap();
    }
}
