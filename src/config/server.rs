use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde(default = "default_backlog")]
    pub backlog: i32,
    #[serde(default = "default_sock_recv_buf")]
    pub sock_recv_buf: usize,
    /// Graceful shutdown timeout in seconds. In-flight connections are given
    /// this long to finish before being forcibly dropped.
    #[serde(default = "default_shutdown_timeout_secs")]
    pub shutdown_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            max_connections: default_max_connections(),
            backlog: default_backlog(),
            sock_recv_buf: default_sock_recv_buf(),
            shutdown_timeout_secs: default_shutdown_timeout_secs(),
        }
    }
}

fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}
fn default_max_connections() -> usize {
    20_000
}
fn default_backlog() -> i32 {
    8192
}
fn default_sock_recv_buf() -> usize {
    256 * 1024
}
fn default_shutdown_timeout_secs() -> u64 {
    30
}
