use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use dotenvy::dotenv;
use tracing::{info, info_span};
use tracing_subscriber::EnvFilter;

use server::config::{Config, LogFormat};
use server::{app, pool};

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
    let config = Config::from_env().expect("config parsing");

    tracing_init(EnvFilter::new(config.log), config.log_format);

    let span = info_span!("pre_launch").entered();
    info!("reading DATABASE_URL env var");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable");
    info!(?database_url, "connecting to database");
    let pool = pool(&database_url).await.expect("database connection");
    info!(?pool, "creating app");
    let app = app(pool);
    span.exit();

    axum::Server::bind(&SocketAddr::new(
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        config.port,
    ))
    .serve(app.into_make_service())
    .await
    .expect("server launch");
}
