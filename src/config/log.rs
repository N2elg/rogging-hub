use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_dir")]
    pub dir: String,
    #[serde(default = "default_log_file_prefix")]
    pub file_prefix: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            dir: default_log_dir(),
            file_prefix: default_log_file_prefix(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_dir() -> String {
    "logs".to_string()
}
fn default_log_file_prefix() -> String {
    "rogginghub".to_string()
}
