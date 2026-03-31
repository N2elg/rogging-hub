mod appender;
mod cleanup;
mod formatter;

use crate::config::{AppenderConfig, LogConfig};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

/// Initialize the logging system from config. Dynamically creates layers
/// for each named appender — just like log4j2 appender configuration.
///
/// Returns guards that **must** be held for the lifetime of the program
/// (dropping them flushes and stops background writer threads).
pub fn init(cfg: &LogConfig) -> Vec<WorkerGuard> {
    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = Vec::new();
    let mut guards: Vec<WorkerGuard> = Vec::new();

    for (name, appender_cfg) in &cfg.appenders {
        let (layer, guard) = match appender_cfg {
            AppenderConfig::Console(c) => appender::build_console_appender(name, c),
            AppenderConfig::RollingFile(f) => {
                cleanup::cleanup_old_files(&f.dir, &f.prefix, f.max_files);
                appender::build_rolling_file_appender(name, f)
            }
        };
        layers.push(layer);
        guards.push(guard);
    }

    let registry = tracing_subscriber::registry();
    registry.with(layers).init();

    guards
}
