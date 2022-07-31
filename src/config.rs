use figment::providers::{Env, Serialized};
use figment::Figment;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub log: String,
    pub log_format: LogFormat,
}

pub fn config() -> figment::error::Result<Config> {
    Figment::from(Serialized::defaults(Config::default()))
        // get the port env var as PORT or MERCURY_PORT
        .merge(Env::raw().only(&["port"]))
        .merge(Env::prefixed("MERCURY_"))
        .extract()
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            log: "info".into(),
            log_format: LogFormat::Json,
        }
    }
}
