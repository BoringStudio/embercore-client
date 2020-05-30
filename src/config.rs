use std::net::SocketAddr;

use config::{Config as RowConfig, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_address: SocketAddr,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let mut config = RowConfig::new();
        config.merge(File::new(path, FileFormat::Yaml))?;
        config.try_into()
    }
}
