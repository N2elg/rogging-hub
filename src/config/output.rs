use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    #[serde(default)]
    pub file: FileOutputConfig,
    #[serde(default)]
    pub sse: SseOutputConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileOutputConfig {
    #[serde(default = "default_output_enabled")]
    pub enabled: bool,
    #[serde(default = "default_output_dir")]
    pub dir: String,
    #[serde(default = "default_output_prefix")]
    pub prefix: String,
    #[serde(default = "default_output_flush_interval_ms")]
    pub flush_interval_ms: u64,
    #[serde(default = "default_output_channel_capacity")]
    pub channel_capacity: usize,
    /// Write mode: "buffered", "mmap", or "direct"
    #[serde(default = "default_write_mode")]
    pub write_mode: String,
    /// Pre-allocated chunk size for mmap mode (default 64MB)
    #[serde(default = "default_mmap_chunk_size")]
    pub mmap_chunk_size: usize,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            file: FileOutputConfig::default(),
            sse: SseOutputConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SseOutputConfig {
    #[serde(default = "default_sse_enabled")]
    pub enabled: bool,
    #[serde(default = "default_sse_listen_addr")]
    pub listen_addr: String,
    /// Broadcast channel capacity — slow clients that fall behind will miss events.
    #[serde(default = "default_sse_channel_capacity")]
    pub channel_capacity: usize,
}

impl Default for SseOutputConfig {
    fn default() -> Self {
        Self {
            enabled: default_sse_enabled(),
            listen_addr: default_sse_listen_addr(),
            channel_capacity: default_sse_channel_capacity(),
        }
    }
}

impl Default for FileOutputConfig {
    fn default() -> Self {
        Self {
            enabled: default_output_enabled(),
            dir: default_output_dir(),
            prefix: default_output_prefix(),
            flush_interval_ms: default_output_flush_interval_ms(),
            channel_capacity: default_output_channel_capacity(),
            write_mode: default_write_mode(),
            mmap_chunk_size: default_mmap_chunk_size(),
        }
    }
}

fn default_output_enabled() -> bool {
    true
}
fn default_output_dir() -> String {
    "output".to_string()
}
fn default_output_prefix() -> String {
    "rogginghub".to_string()
}
fn default_output_flush_interval_ms() -> u64 {
    1000
}
fn default_output_channel_capacity() -> usize {
    8192
}
fn default_write_mode() -> String {
    "mmap".to_string()
}
fn default_mmap_chunk_size() -> usize {
    64 * 1024 * 1024 // 64 MB
}

fn default_sse_enabled() -> bool {
    false
}
fn default_sse_listen_addr() -> String {
    "0.0.0.0:8081".to_string()
}
fn default_sse_channel_capacity() -> usize {
    4096
}
