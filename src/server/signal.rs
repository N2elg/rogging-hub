use tokio_util::sync::CancellationToken;
use tracing::info;

/// Spawn a background task that cancels the token on SIGINT or SIGTERM.
pub fn spawn_signal_handler() -> CancellationToken {
    let shutdown = CancellationToken::new();
    let token = shutdown.clone();
    tokio::spawn(async move {
        wait_for_signal().await;
        token.cancel();
    });
    shutdown
}

async fn wait_for_signal() {
    use tokio::signal;

    let ctrl_c = signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");

        tokio::select! {
            _ = ctrl_c => info!("Received SIGINT"),
            _ = sigterm.recv() => info!("Received SIGTERM"),
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
        info!("Received SIGINT");
    }
}
