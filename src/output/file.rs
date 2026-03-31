use crate::config::FileOutputConfig;
use crate::output::writer::{self, OutputWriter};
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info, warn};

/// Runs the file output task. Receives NDJSON lines from parsers and writes
/// them via the configured writer backend (buffered / mmap / direct).
pub async fn run_file_output(
    cfg: FileOutputConfig,
    mut rx: Receiver<Vec<u8>>,
) {
    info!(
        dir = %cfg.dir,
        prefix = %cfg.prefix,
        write_mode = %cfg.write_mode,
        "File output task started"
    );

    let flush_interval = tokio::time::Duration::from_millis(cfg.flush_interval_ms);
    let mut flush_timer = tokio::time::interval(flush_interval);
    flush_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut current_writer: Option<Box<dyn OutputWriter>> = None;
    let mut total_written: u64 = 0;
    let mut total_bytes: u64 = 0;

    loop {
        tokio::select! {
            biased;

            msg = rx.recv() => {
                match msg {
                    Some(data) => {
                        let today = today_str();
                        let w = match ensure_writer(
                            &mut current_writer, &cfg, &today,
                        ) {
                            Ok(w) => w,
                            Err(e) => {
                                error!(error = %e, "Failed to open output file");
                                continue;
                            }
                        };
                        match w.write_record(&data) {
                            Ok(n) => {
                                total_written += 1;
                                total_bytes += n as u64;
                                debug!(total_written, "Wrote JSON to file");
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to write record");
                            }
                        }
                    }
                    None => {
                        // All senders dropped — drain complete.
                        if let Some(w) = current_writer.take() {
                            if let Err(e) = w.close() {
                                warn!(error = %e, "Failed to close output file");
                            }
                        }
                        info!(total_written, total_bytes, "File output task finished");
                        return;
                    }
                }
            }

            _ = flush_timer.tick() => {
                if let Some(ref mut w) = current_writer {
                    if let Err(e) = w.flush() {
                        warn!(error = %e, "Flush failed");
                    }
                }
            }
        }
    }
}

fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Ensure we have a writer open for today's date. Rolls to a new file
/// when the date changes, closing the previous writer properly.
fn ensure_writer<'a>(
    current: &'a mut Option<Box<dyn OutputWriter>>,
    cfg: &FileOutputConfig,
    today: &str,
) -> std::io::Result<&'a mut Box<dyn OutputWriter>> {
    let needs_new = match current {
        Some(w) => w.date() != today,
        None => true,
    };

    if needs_new {
        // Close previous writer (flushes, truncates mmap, etc.).
        if let Some(old) = current.take() {
            if let Err(e) = old.close() {
                warn!(error = %e, "Failed to close previous output file");
            }
        }

        let w = writer::create_writer(
            &cfg.write_mode,
            &cfg.dir,
            &cfg.prefix,
            today,
            cfg.mmap_chunk_size,
        )?;
        *current = Some(w);
    }

    Ok(current.as_mut().unwrap())
}
