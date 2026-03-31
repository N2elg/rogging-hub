use bytes::BytesMut;
use tokio::sync::{broadcast, mpsc::{Receiver, Sender}};
use tracing::{debug, info, warn};

pub async fn run_parser(
    mut rx: Receiver<BytesMut>,
    output_tx: Option<Sender<Vec<u8>>>,
    sse_tx: Option<broadcast::Sender<bytes::Bytes>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut parsed: u64 = 0;
    let mut errors: u64 = 0;

    debug!("Parser task started");

    while let Some(mut obj) = rx.recv().await {
        debug!(len = obj.len(), "Parsing object");
        match simd_json::from_slice::<simd_json::OwnedValue>(obj.as_mut()) {
            Ok(json) => {
                parsed += 1;
                debug!(%json, "Parsed JSON");

                let serialized = json.to_string().into_bytes();

                // Send to file output if enabled.
                if let Some(ref tx) = output_tx {
                    if tx.send(serialized.clone()).await.is_err() {
                        warn!("File output channel closed, dropping message");
                    }
                }

                // Broadcast to SSE clients (no-op if no subscribers).
                if let Some(ref tx) = sse_tx {
                    let _ = tx.send(bytes::Bytes::from(serialized));
                }
            }
            Err(e) => {
                errors += 1;
                warn!(error = %e, len = obj.len(), "Failed to parse JSON");
            }
        }
    }

    info!(parsed, errors, "Parser task finished");
    Ok(())
}
