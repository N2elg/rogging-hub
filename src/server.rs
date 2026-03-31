use crate::config::{Config, ServerConfig};
use crate::io_handler;
use crate::output;
use socket2::{Domain, Protocol, Socket, Type};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub async fn serve(cfg: Config, runtimes: &crate::runtime::Runtimes) {
    let listener = create_listener(&cfg.server).expect("Failed to create listener");
    info!(addr = %cfg.server.listen_addr, "Listening for JSON streams");

    // Shutdown signal — triggered by SIGTERM or SIGINT.
    let shutdown = CancellationToken::new();
    let shutdown_signal = shutdown.clone();
    tokio::spawn(async move {
        wait_for_signal().await;
        shutdown_signal.cancel();
    });

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

    let sse_tx = if cfg.output.sse.enabled {
        let sse_cfg = cfg.output.sse.clone();
        Some(output::run_sse_output(sse_cfg).await)
    } else {
        info!("SSE output disabled");
        None
    };

    let sem = Arc::new(Semaphore::new(cfg.server.max_connections));
    let conn_shutdown = shutdown.clone();

    // ── Accept loop ──
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
                        let active = cfg.server.max_connections - sem.available_permits();
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
    let timeout_secs = cfg.server.shutdown_timeout_secs;
    let max = cfg.server.max_connections;
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
    ).await.is_err() {
        let remaining = max - sem.available_permits();
        warn!(remaining, "Drain timeout reached, dropping remaining connections");
    } else {
        info!("All in-flight connections drained");
    }

    // ── Flush output ──
    // Drop output sender so the file output task can drain and close.
    drop(output_tx);
    info!("Output channel closed, waiting for file output to flush");

    // Give the output task time to flush.
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    info!("Graceful shutdown complete");
}

/// Wait for SIGINT (Ctrl+C) or SIGTERM.
async fn wait_for_signal() {
    use tokio::signal;

    let ctrl_c = signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut sigterm =
            signal::unix::signal(signal::unix::SignalKind::terminate())
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

fn create_listener(server: &ServerConfig) -> std::io::Result<TcpListener> {
    let addr: std::net::SocketAddr = server.listen_addr.parse().unwrap();
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.set_nodelay(true)?;
    socket.set_nonblocking(true)?;
    socket.set_recv_buffer_size(server.sock_recv_buf)?;

    socket.bind(&addr.into())?;
    socket.listen(server.backlog)?;

    let std_listener: std::net::TcpListener = socket.into();
    TcpListener::from_std(std_listener)
}
