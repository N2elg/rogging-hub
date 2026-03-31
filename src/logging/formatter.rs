use std::fmt;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

/// Configurable log pattern akin to log4j2's `PatternLayout`.
///
/// Supported placeholders:
///   {timestamp}  → `2026-04-01 12:00:00.123`
///   {level}      → `INFO ` (5-char padded)
///   {module}     → `server::accept`
///   {message}    → field values + message body
///
/// Everything else in the pattern string is emitted verbatim.
pub(crate) struct PatternFormatter {
    pattern: String,
    ansi: bool,
}

impl PatternFormatter {
    pub(crate) fn new(pattern: &str, ansi: bool) -> Self {
        Self {
            pattern: pattern.to_string(),
            ansi,
        }
    }
}

impl<S, N> FormatEvent<S, N> for PatternFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let now = chrono::Local::now();
        let meta = event.metadata();
        let level = *meta.level();
        let module = meta.module_path().unwrap_or("-");

        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let level_str = if self.ansi {
            let (cs, ce) = level_color(level);
            format!("{cs}{level:<5}{ce}")
        } else {
            format!("{level:<5}")
        };

        // Replace placeholders — walk through the pattern.
        let formatted = self
            .pattern
            .replace("{timestamp}", &timestamp)
            .replace("{level}", &level_str)
            .replace("{module}", module);

        // Split on {message} — write prefix, then fields, then suffix.
        if let Some((before, after)) = formatted.split_once("{message}") {
            write!(writer, "{before}")?;
            ctx.field_format().format_fields(writer.by_ref(), event)?;
            write!(writer, "{after}")?;
        } else {
            // No {message} placeholder — write pattern then fields.
            write!(writer, "{formatted} ")?;
            ctx.field_format().format_fields(writer.by_ref(), event)?;
        }

        writeln!(writer)
    }
}

/// ANSI color codes for log levels.
fn level_color(level: Level) -> (&'static str, &'static str) {
    match level {
        Level::ERROR => ("\x1b[31m", "\x1b[0m"), // red
        Level::WARN => ("\x1b[33m", "\x1b[0m"),  // yellow
        Level::INFO => ("\x1b[32m", "\x1b[0m"),  // green
        Level::DEBUG => ("\x1b[36m", "\x1b[0m"), // cyan
        Level::TRACE => ("\x1b[90m", "\x1b[0m"), // gray
    }
}
