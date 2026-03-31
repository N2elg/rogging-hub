use RoggingHub::config::{Config, LogConfig};

#[test]
fn default_config_has_expected_values() {
    let cfg = Config::default();
    assert_eq!(cfg.server.listen_addr, "0.0.0.0:8080");
    assert_eq!(cfg.server.max_connections, 20_000);
    assert_eq!(cfg.server.backlog, 8192);
    assert_eq!(cfg.server.sock_recv_buf, 256 * 1024);
    assert_eq!(cfg.server.shutdown_timeout_secs, 30);
    assert!(cfg.output.file.enabled);
    assert_eq!(cfg.output.file.write_mode, "mmap");
    assert!(!cfg.output.sse.enabled);
    assert_eq!(cfg.runtime.output_threads, 2);
}

#[test]
fn default_log_config_has_expected_values() {
    let log_cfg = LogConfig::default();
    assert_eq!(log_cfg.root_level, "info");
    assert!(log_cfg.appenders.contains_key("console"));
    assert!(log_cfg.appenders.contains_key("file"));
}

#[test]
fn parse_empty_toml_gives_defaults() {
    let cfg: Config = toml::from_str("").unwrap();
    assert_eq!(cfg.server.listen_addr, "0.0.0.0:8080");
    assert_eq!(cfg.server.max_connections, 20_000);
}

#[test]
fn parse_partial_toml_overrides_specified() {
    let toml_str = r#"
[server]
listen_addr = "127.0.0.1:9090"
max_connections = 5000

[output.file]
write_mode = "direct"
"#;
    let cfg: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.server.listen_addr, "127.0.0.1:9090");
    assert_eq!(cfg.server.max_connections, 5000);
    assert_eq!(cfg.server.backlog, 8192);
    assert_eq!(cfg.output.file.write_mode, "direct");
    assert!(cfg.output.file.enabled);
}

#[test]
fn parse_full_toml() {
    let toml_str = r#"
[server]
listen_addr = "0.0.0.0:3000"
max_connections = 100
backlog = 128
sock_recv_buf = 8192
shutdown_timeout_secs = 10

[output.file]
enabled = false
dir = "/tmp/output"
prefix = "myapp"
flush_interval_ms = 500
channel_capacity = 1024
write_mode = "buffered"
mmap_chunk_size = 1048576

[output.sse]
enabled = true
listen_addr = "0.0.0.0:9090"
channel_capacity = 2048

[runtime]
parser_threads = 8
output_threads = 4
"#;
    let cfg: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.server.listen_addr, "0.0.0.0:3000");
    assert_eq!(cfg.server.max_connections, 100);
    assert_eq!(cfg.server.shutdown_timeout_secs, 10);
    assert!(!cfg.output.file.enabled);
    assert_eq!(cfg.output.file.write_mode, "buffered");
    assert_eq!(cfg.output.file.mmap_chunk_size, 1048576);
    assert!(cfg.output.sse.enabled);
    assert_eq!(cfg.output.sse.channel_capacity, 2048);
    assert_eq!(cfg.runtime.parser_threads, 8);
    assert_eq!(cfg.runtime.output_threads, 4);
}

#[test]
fn parse_full_log_toml() {
    let toml_str = r#"
root_level = "debug"

[appenders.console]
kind = "console"
level = "warn"

[appenders.file]
kind = "rolling_file"
level = "debug"
dir = "/tmp/logs"
prefix = "myapp"
roll = "hourly"
max_files = 10
"#;
    let log_cfg: LogConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(log_cfg.root_level, "debug");
    assert_eq!(log_cfg.appenders.len(), 2);
    use RoggingHub::config::{AppenderConfig, RollPolicy};
    match &log_cfg.appenders["file"] {
        AppenderConfig::RollingFile(f) => {
            assert_eq!(f.dir, "/tmp/logs");
            assert_eq!(f.prefix, "myapp");
            assert_eq!(f.roll, RollPolicy::Hourly);
            assert_eq!(f.max_files, 10);
        }
        _ => panic!("Expected RollingFile appender"),
    }
}

#[test]
fn parse_invalid_toml_errors() {
    let result = toml::from_str::<Config>("invalid [[[ toml");
    assert!(result.is_err());
}
