use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Result;
use server::app;
use server::config::{LogFormat, CONFIG};
use tracing_subscriber::EnvFilter;

fn tracing_init() {
    let subscriber_builder = tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&CONFIG.log));
    match CONFIG.log_format {
        LogFormat::Json => subscriber_builder.json().init(),
        LogFormat::Pretty => subscriber_builder.pretty().init(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_init();

    let app = app().await?;

    axum::Server::bind(&SocketAddr::new(
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        CONFIG.port,
    ))
    .serve(app.into_make_service())
    .await?;

    Ok(())
}
