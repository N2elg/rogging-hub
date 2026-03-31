# RoggingHub

A high-performance, C10K-ready log aggregation hub written in Rust. Designed as a drop-in replacement for Logstash — receives JSON log streams over TCP, parses them with SIMD-accelerated JSON parsing, and outputs to rolling files or SSE.

Core pipeline settings are loaded from `config.toml`, while logging is configured independently through `logging.toml`.

## Features

- **C10K ready** — handles 20,000+ concurrent TCP connections via Tokio async runtime
- **SIMD JSON parsing** — uses `simd-json` for 2–4x faster parsing than `serde_json`
- **Zero-copy framing** — brace-depth based JSON framer with sticky/half-packet handling
- **3 write modes** — buffered, mmap (recommended), or direct I/O (page cache bypass)
- **Separated thread pools** — IO, parser, and output each run on dedicated runtimes
- **SSE output** — real-time Server-Sent Events stream with gzip/brotli/zstd compression
- **log4j2-style logging** — named appenders, per-appender levels, configurable patterns, rolling files
- **Graceful shutdown** — drains in-flight connections, flushes outputs on SIGINT/SIGTERM

## Architecture

```
 TCP Clients (Logstash-format JSON)
       │
       ▼
┌──────────────────────────────────────────────────────────────────┐
│  IO Runtime                                                      │
│  ┌────────────┐   ┌────────────┐   ┌────────────┐               │
│  │ TCP Accept  │──▶│ IO Reader  │   │ IO Reader  │  ...          │
│  │ (semaphore) │   │ + Framer   │   │ + Framer   │               │
│  └────────────┘   └─────┬──────┘   └─────┬──────┘               │
│                         │                │                       │
└─────────────────────────┼────────────────┼───────────────────────┘
                 mpsc     │       mpsc     │
                 channel  │       channel  │
┌─────────────────────────┼────────────────┼───────────────────────┐
│  Parser Runtime         ▼                ▼                       │
│  ┌────────────┐   ┌────────────┐                                 │
│  │  Parser 1  │   │  Parser 2  │   ...                           │
│  └──────┬─────┘   └──────┬─────┘                                 │
│         │                │                                       │
└─────────┼────────────────┼───────────────────────────────────────┘
          │  mpsc + broadcast                                       
          │  channels                                               
┌─────────▼────────────────▼───────────────────────────────────────┐
│  Output Runtime                                                   │
│  ┌──────────────────┐   ┌──────────────────┐                     │
│  │ File Writer      │   │ SSE Server       │                     │
│  │ (mmap/direct/buf)│   │ (axum + compress)│                     │
│  └──────────────────┘   └──────────────────┘                     │
└──────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
RoggingHub/
├── Cargo.toml              # Dependencies and build profile
├── config.toml             # Server, output, and runtime config
├── logging.toml            # log4j2-style logging config
├── .cargo/
│   └── config.toml         # Rust build flags (target-cpu=native)
├── src/
│   ├── main.rs             # Entry point
│   ├── lib.rs              # Library root (re-exports modules)
│   ├── config/
│   │   ├── mod.rs          # Config loader
│   │   ├── server.rs       # ServerConfig
│   │   ├── log.rs          # LogConfig (logging.toml)
│   │   ├── output.rs       # FileOutputConfig, SseOutputConfig
│   │   └── runtime.rs      # RuntimeConfig
│   ├── logging/
│   │   ├── mod.rs          # Dynamic logging init from named appenders
│   │   ├── formatter.rs    # Pattern formatter for log layouts
│   │   ├── appender.rs     # Console + rolling file appender builders
│   │   └── cleanup.rs      # Old log file cleanup by max_files
│   ├── server/
│   │   ├── mod.rs          # Server orchestration
│   │   ├── accept.rs       # TCP accept loop with backpressure
│   │   ├── listener.rs     # socket2 TCP listener tuning
│   │   └── signal.rs       # SIGINT/SIGTERM handler
│   ├── io_handler/
│   │   ├── mod.rs          # Connection handler orchestration
│   │   ├── framer.rs       # Brace-depth JSON framer (state machine)
│   │   ├── io.rs           # Async TCP reader
│   │   └── parser.rs       # simd-json parser task
│   ├── runtime/
│   │   ├── mod.rs          # Builds and owns the three Tokio runtimes
│   │   ├── io.rs           # IO runtime builder
│   │   ├── parser.rs       # Parser runtime builder
│   │   └── output.rs       # Output runtime builder
│   └── output/
│       ├── mod.rs          # Output module root
│       ├── file.rs         # Date-based NDJSON file output task
│       ├── sse.rs          # SSE HTTP server (axum)
│       └── writer/
│           ├── mod.rs      # OutputWriter trait + factory
│           ├── buffered.rs # BufWriter backend
│           ├── mmap.rs     # Memory-mapped backend
│           └── direct.rs   # Direct I/O backend
└── tests/
    ├── test_config.rs      # config.toml + logging.toml parsing tests
    ├── test_framer.rs      # JSON framer edge cases
    ├── test_parser.rs      # Parser integration tests
    └── test_writer.rs      # Writer backend tests
```

## Build

**Requirements:** Rust 1.85+ (edition 2024)

```bash
# Debug build
cargo build

# Release build (optimized, LTO, native SIMD instructions)
cargo build --release

# The binary is at:
#   debug:   target/debug/RoggingHub
#   release: target/release/RoggingHub
```

## Run

```bash
# Copy config files to the working directory
cp config.toml logging.toml /path/to/deploy/

# Run the binary
./target/release/RoggingHub

# Or run directly with cargo
cargo run --release
```

At startup RoggingHub loads `logging.toml` and `config.toml` separately. If either file is missing, that part of the configuration falls back to built-in defaults.

Before running in production, increase the file descriptor limit:

```bash
ulimit -n 65536
```

## Configuration

`config.toml` contains server, output, and runtime settings. `logging.toml` contains the logging pipeline and is no longer embedded inside `config.toml`.

### config.toml

```toml
[server]
listen_addr = "0.0.0.0:8080"
max_connections = 20000
backlog = 8192
sock_recv_buf = 262144
shutdown_timeout_secs = 30

[output.file]
enabled = true
dir = "output"
prefix = "rogginghub"
write_mode = "mmap"           # "buffered" | "mmap" | "direct"
mmap_chunk_size = 67108864    # 64 MB
flush_interval_ms = 1000
channel_capacity = 8192

[output.sse]
enabled = false
listen_addr = "0.0.0.0:8081"
channel_capacity = 4096

[runtime]
parser_threads = 4
output_threads = 2
```

### logging.toml

```toml
root_level = "info"

[appenders.console]
kind = "console"
level = "info"
ansi = true
pattern = "{timestamp} [{level}] [{module}] {message}"

[appenders.file]
kind = "rolling_file"
level = "debug"
dir = "logs"
prefix = "rogginghub"
roll = "daily"
max_files = 30
pattern = "{timestamp} [{level}] [{module}] {message}"
```

Notes:

- Appenders are declared as named tables under `[appenders.<name>]`.
- Supported appender kinds are `console` and `rolling_file`.
- Rolling file policies are `daily`, `hourly`, and `never`.
- Supported pattern placeholders are `{timestamp}`, `{level}`, `{module}`, and `{message}`.
- If `logging.toml` is absent, RoggingHub uses built-in defaults for a console appender and a rolling file appender.

## Test

```bash
# Run all tests
cargo test

# Send a test JSON stream
cargo run --release &
echo '{"host":"web01","message":"hello","@timestamp":"2026-04-01T00:00:00Z"}' | nc localhost 8080

# Connect to SSE stream (if enabled)
curl -N http://localhost:8081/events
```

## License

See [LICENSE](LICENSE) for details.
