use axum::response::IntoResponse;
use axum::routing::{get, get_service};
use axum::Router;
use figment::providers::{Env, Serialized};
use figment::Figment;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum LogFormat {
    Json,
    Pretty,
}

#[derive(Deserialize, Serialize)]
struct Config {
    port: u16,
    log: String,
    log_format: LogFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3000,
            log: "server=debug,tower_http=debug".into(),
            log_format: LogFormat::Json,
        }
    }
}

fn tracing_init(env_filter: EnvFilter, log_format: LogFormat) {
    let sub = tracing_subscriber::fmt().with_env_filter(env_filter);
    match log_format {
        LogFormat::Json => sub.json().init(),
        LogFormat::Pretty => sub.pretty().init(),
    }
}

fn app() -> Router {
    Router::new().route("/", get(server::hello))
}

#[tokio::main]
async fn main() {
    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Env::raw().only(&["port"]))
        .merge(Env::prefixed("MERCURY_"))
        .extract()
        .expect("environment config parsing");

    tracing_init(EnvFilter::new(config.log), config.log_format);

    let app = Router::new()
        .nest("/api", app())
        .fallback(get_service(ServeDir::new("static")).handle_error(handle_io_error))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&SocketAddr::new(
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        config.port,
    ))
    .serve(app.into_make_service())
    .await
    .expect("server launch");
}

async fn handle_io_error(error: io::Error) -> impl IntoResponse {
    error!(?error);
    StatusCode::INTERNAL_SERVER_ERROR
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn hello() {
        let app = app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, World!");
    }
}
