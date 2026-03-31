use crate::config::LogConfig;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initialize non-blocking tracing to console + daily rolling file.
/// Returns guards that must be held for the lifetime of the program.
pub fn init(cfg: &LogConfig) -> (WorkerGuard, WorkerGuard) {
    let (console_nb, console_guard) = tracing_appender::non_blocking(std::io::stdout());
    let file_appender = tracing_appender::rolling::daily(&cfg.dir, &cfg.file_prefix);
    let (file_nb, file_guard) = tracing_appender::non_blocking(file_appender);

    let env_filter =
        EnvFilter::try_new(&cfg.level).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_writer(console_nb),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_ansi(false)
                .with_writer(file_nb),
        )
        .init();

    (console_guard, file_guard)
}
