mod accept;
mod listener;
mod signal;

use crate::config::Config;
use crate::output;
use crate::runtime::Runtimes;
use tokio::sync::mpsc;
use tracing::info;

pub async fn serve(cfg: Config, runtimes: &Runtimes) {
    let listener = listener::create_listener(&cfg.server).expect("Failed to create listener");
    info!(addr = %cfg.server.listen_addr, "Listening for JSON streams");

    // Shutdown token — triggered by SIGTERM / SIGINT.
    let shutdown = signal::spawn_signal_handler();

    // File output channel.
    let output_tx = if cfg.output.file.enabled {
        let (tx, rx) = mpsc::channel::<Vec<u8>>(cfg.output.file.channel_capacity);
        let file_cfg = cfg.output.file.clone();
        runtimes.output.spawn(async move {
            output::run_file_output(file_cfg, rx).await;
        });
        Some(tx)
    } else {
        info!("File output disabled");
        None
    };

    // SSE broadcast channel.
    let sse_tx = if cfg.output.sse.enabled {
        let sse_cfg = cfg.output.sse.clone();
        Some(output::run_sse_output(sse_cfg).await)
    } else {
        info!("SSE output disabled");
        None
    };

    // Accept loop + drain + flush.
    accept::run_accept_loop(
        &cfg.server,
        listener,
        shutdown,
        output_tx,
        sse_tx,
        runtimes,
    )
    .await;
}
