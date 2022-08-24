use figment::providers::{Env, Serialized};
use figment::Figment;
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
}

impl Config {
    pub fn from_env() -> figment::error::Result<Config> {
        Figment::from(Serialized::defaults(Config::default()))
            // get the port env var as PORT or MERCURY_PORT
            .merge(Env::raw().only(&["port"]))
            .merge(Env::prefixed("MERCURY_"))
            .extract()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            log: "warn".to_owned(),
            log_format: LogFormat::Json,
        }
    }
}
