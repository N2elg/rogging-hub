use crate::config::SseOutputConfig;
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use tower_http::compression::CompressionLayer;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::{info, warn};

/// Shared state for the SSE HTTP server.
#[derive(Clone)]
struct SseState {
    tx: Arc<broadcast::Sender<bytes::Bytes>>,
}

/// Start the SSE HTTP server and return the broadcast sender.
/// Parsed JSON events sent to the returned sender will be streamed
/// to all connected SSE clients.
pub async fn run_sse_output(cfg: SseOutputConfig) -> broadcast::Sender<bytes::Bytes> {
    let (tx, _) = broadcast::channel(cfg.channel_capacity);
    let sender = tx.clone();

    let state = SseState {
        tx: Arc::new(tx),
    };

    let compression = CompressionLayer::new()
        .br(true)
        .gzip(true)
        .zstd(true);

    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(compression)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.listen_addr)
        .await
        .expect("Failed to bind SSE listener");

    info!(addr = %cfg.listen_addr, "SSE output server started on /events");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            warn!(error = %e, "SSE server error");
        }
    });

    sender
}

async fn sse_handler(
    State(state): State<SseState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    info!("SSE client connected");

    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(data) => {
            let text = String::from_utf8_lossy(&data).into_owned();
            Some(Ok(Event::default().data(text)))
        }
        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
            warn!(missed = n, "SSE client lagged, skipped events");
            None
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
