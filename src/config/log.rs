use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// log4j2-style logging config.
///
/// Loaded from `logging.toml` (separate from `config.toml`).
///
/// ```toml
/// root_level = "info"
///
/// [appenders.console]
/// kind = "console"
/// level = "info"
/// ansi = true
/// pattern = "{timestamp} [{level}] [{module}] {message}"
///
/// [appenders.file]
/// kind = "rolling_file"
/// level = "debug"
/// dir = "logs"
/// prefix = "rogginghub"
/// roll = "daily"
/// max_files = 30
/// pattern = "{timestamp} [{level}] [{module}] {message}"
/// ```
#[derive(Debug, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_root_level")]
    pub root_level: String,
    #[serde(default = "default_appenders")]
    pub appenders: HashMap<String, AppenderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum AppenderConfig {
    #[serde(rename = "console")]
    Console(ConsoleAppender),
    #[serde(rename = "rolling_file")]
    RollingFile(RollingFileAppender),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsoleAppender {
    #[serde(default = "default_root_level")]
    pub level: String,
    #[serde(default = "default_true")]
    pub ansi: bool,
    #[serde(default = "default_pattern")]
    pub pattern: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RollingFileAppender {
    #[serde(default = "default_root_level")]
    pub level: String,
    #[serde(default = "default_log_dir")]
    pub dir: String,
    #[serde(default = "default_log_prefix")]
    pub prefix: String,
    #[serde(default = "default_roll")]
    pub roll: RollPolicy,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    #[serde(default = "default_pattern")]
    pub pattern: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RollPolicy {
    Daily,
    Hourly,
    Never,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            root_level: default_root_level(),
            appenders: default_appenders(),
        }
    }
}

impl LogConfig {
    pub fn load() -> Self {
        let path = Path::new("logging.toml");
        if path.exists() {
            let content = std::fs::read_to_string(path).expect("Failed to read logging.toml");
            toml::from_str(&content).expect("Failed to parse logging.toml")
        } else {
            Self::default()
        }
    }
}

fn default_root_level() -> String {
    "info".to_string()
}
fn default_true() -> bool {
    true
}
fn default_pattern() -> String {
    "{timestamp} [{level}] [{module}] {message}".to_string()
}
fn default_log_dir() -> String {
    "logs".to_string()
}
fn default_log_prefix() -> String {
    "rogginghub".to_string()
}
fn default_roll() -> RollPolicy {
    RollPolicy::Daily
}
fn default_max_files() -> usize {
    30
}

fn default_appenders() -> HashMap<String, AppenderConfig> {
    let mut map = HashMap::new();
    map.insert(
        "console".to_string(),
        AppenderConfig::Console(ConsoleAppender {
            level: "info".to_string(),
            ansi: true,
            pattern: default_pattern(),
        }),
    );
    map.insert(
        "file".to_string(),
        AppenderConfig::RollingFile(RollingFileAppender {
            level: "debug".to_string(),
            dir: "logs".to_string(),
            prefix: "rogginghub".to_string(),
            roll: RollPolicy::Daily,
            max_files: 30,
            pattern: default_pattern(),
        }),
    );
    map
}
