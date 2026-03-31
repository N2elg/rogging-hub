mod log;
mod output;
mod runtime;
mod server;

pub use log::LogConfig;
pub use output::{FileOutputConfig, OutputConfig, SseOutputConfig};
pub use runtime::RuntimeConfig;
pub use server::ServerConfig;

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            log: LogConfig::default(),
            output: OutputConfig::default(),
            runtime: RuntimeConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Path::new("config.toml");
        if path.exists() {
            let content = std::fs::read_to_string(path).expect("Failed to read config.toml");
            toml::from_str(&content).expect("Failed to parse config.toml")
        } else {
            Self::default()
        }
    }
}
