use std::env;
use std::str::FromStr;
use std::time::Duration;

use crate::error::{Result, ServerError};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub max_header_bytes: usize,
    pub max_body_bytes: usize,
    pub max_requests_per_connection: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 16,
            read_timeout: Duration::from_secs(5),
            write_timeout: Duration::from_secs(5),
            max_header_bytes: 16 * 1024,
            max_body_bytes: 1024 * 1024,
            max_requests_per_connection: 100,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();

        if let Some(host) = read_env::<String>("HOST")? {
            config.host = host;
        }
        if let Some(port) = read_env::<u16>("PORT")? {
            config.port = port;
        }
        if let Some(workers) = read_env::<usize>("WORKERS")? {
            config.workers = workers;
        }

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.workers == 0 {
            return Err(ServerError::Config("WORKERS must be at least 1".into()));
        }
        Ok(())
    }
}

fn read_env<T: FromStr>(key: &str) -> Result<Option<T>> {
    match env::var(key) {
        Ok(raw) => raw
            .parse::<T>()
            .map(Some)
            .map_err(|_| ServerError::Config(format!("invalid value for {key}: {raw:?}"))),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(ServerError::Config(format!("{key}: {e}"))),
    }
}
