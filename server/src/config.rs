use dotenvy::dotenv;
use figment::providers::{Env, Serialized};
use figment::Figment;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub log: String,
    pub log_format: LogFormat,
    pub database_url: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv().ok();
    Figment::new()
        // default values
        .join(Serialized::default("port", 8080))
        .join(Serialized::default("log_format", LogFormat::Json))
        // get the database_url and port config values with or without the MERCURY_ prefix
        .merge(Env::raw().only(&["port", "database_url"]))
        .merge(Env::prefixed("MERCURY_"))
        .extract()
        .expect("config")
});
