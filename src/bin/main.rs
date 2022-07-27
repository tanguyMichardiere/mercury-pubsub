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
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[sqlx_database_tester::test(pool(variable = "pool"))]
    async fn hello() {
        let app = app(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/hello")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, Mercury!");
    }
}
