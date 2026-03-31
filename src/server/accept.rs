use crate::config::ServerConfig;
use crate::io_handler;
use crate::runtime::Runtimes;
use bytes::Bytes;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub async fn run_accept_loop(
    server_cfg: &ServerConfig,
    listener: TcpListener,
    shutdown: CancellationToken,
    output_tx: Option<mpsc::Sender<Vec<u8>>>,
    sse_tx: Option<broadcast::Sender<Bytes>>,
    runtimes: &Runtimes,
) {
    let sem = Arc::new(Semaphore::new(server_cfg.max_connections));
    let conn_shutdown = shutdown.clone();

    loop {
        let permit = match sem.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => break,
        };

        tokio::select! {
            biased;

            _ = shutdown.cancelled() => {
                info!("Shutdown signal received, stopping accept loop");
                drop(permit);
                break;
            }

            result = listener.accept() => {
                match result {
                    Ok((stream, peer_addr)) => {
                        let active = server_cfg.max_connections - sem.available_permits();
                        info!(%peer_addr, active_connections = active, "Accepted connection");
                        let out_tx = output_tx.clone();
                        let sse_out = sse_tx.clone();
                        let p_rt = runtimes.parser.clone();
                        let conn_token = conn_shutdown.clone();
                        tokio::spawn(async move {
                            if let Err(e) = io_handler::handle_json_stream(
                                stream, out_tx, sse_out, p_rt, conn_token,
                            ).await {
                                warn!(%peer_addr, error = %e, "Connection error");
                            }
                            drop(permit);
                        });
                    }
                    Err(e) => {
                        error!(error = %e, "Accept failed");
                        drop(permit);
                    }
                }
            }
        }
    }

    // ── Drain in-flight connections ──
    drain_connections(sem, server_cfg).await;

    // ── Flush output ──
    drop(output_tx);
    info!("Output channel closed, waiting for file output to flush");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    info!("Graceful shutdown complete");
}

async fn drain_connections(sem: Arc<Semaphore>, server_cfg: &ServerConfig) {
    let timeout_secs = server_cfg.shutdown_timeout_secs;
    let max = server_cfg.max_connections;
    info!(timeout_secs, "Waiting for in-flight connections to drain");

    let drain_done = async {
        loop {
            if sem.available_permits() >= max {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    };

    if tokio::time::timeout(
        tokio::time::Duration::from_secs(timeout_secs),
        drain_done,
    )
    .await
    .is_err()
    {
        let remaining = max - sem.available_permits();
        warn!(remaining, "Drain timeout reached, dropping remaining connections");
    } else {
        info!("All in-flight connections drained");
    }
}
