use crate::config::{ConsoleAppender, RollingFileAppender, RollPolicy};
use super::formatter::PatternFormatter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, Registry};

/// Build a boxed `Layer` + guard for a console appender.
pub(crate) fn build_console_appender(
    name: &str,
    cfg: &ConsoleAppender,
) -> (Box<dyn Layer<Registry> + Send + Sync>, WorkerGuard) {
    let (nb, guard) = tracing_appender::non_blocking(std::io::stdout());
    let filter = EnvFilter::try_new(&cfg.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let layer = tracing_subscriber::fmt::layer()
        .event_format(PatternFormatter::new(&cfg.pattern, cfg.ansi))
        .with_writer(nb)
        .with_filter(filter)
        .boxed();

    tracing::info!(appender = name, level = %cfg.level, "Console appender created");
    (layer, guard)
}

/// Build a boxed `Layer` + guard for a rolling file appender.
pub(crate) fn build_rolling_file_appender(
    name: &str,
    cfg: &RollingFileAppender,
) -> (Box<dyn Layer<Registry> + Send + Sync>, WorkerGuard) {
    let _ = std::fs::create_dir_all(&cfg.dir);

    let file_appender = match cfg.roll {
        RollPolicy::Daily => tracing_appender::rolling::daily(&cfg.dir, &cfg.prefix),
        RollPolicy::Hourly => tracing_appender::rolling::hourly(&cfg.dir, &cfg.prefix),
        RollPolicy::Never => tracing_appender::rolling::never(&cfg.dir, &cfg.prefix),
    };

    let (nb, guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::try_new(&cfg.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let layer = tracing_subscriber::fmt::layer()
        .event_format(PatternFormatter::new(&cfg.pattern, false))
        .with_writer(nb)
        .with_filter(filter)
        .boxed();

    tracing::info!(
        appender = name,
        dir = %cfg.dir,
        prefix = %cfg.prefix,
        roll = ?cfg.roll,
        level = %cfg.level,
        "Rolling file appender created"
    );
    (layer, guard)
}
