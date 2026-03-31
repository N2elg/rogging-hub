use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use super::framer::JsonFramer;
use tracing::{debug, info, warn};

/// Read buffer capacity per connection (64 KB).
const BUF_CAPACITY: usize = 64 * 1024;
/// Max single JSON object size (1 MB) — prevents OOM from malformed streams.
const MAX_JSON_LEN: usize = 1024 * 1024;

/// Reads raw bytes from a TCP stream, frames complete JSON objects using brace-depth
/// tracking, and sends them to the parser task via a bounded channel.
///
/// ## Nagle's Algorithm & TCP_NODELAY
///
/// Nagle's algorithm (RFC 896) is a TCP optimization that buffers small outgoing
/// segments and coalesces them into larger ones to reduce the number of packets on
/// the wire. It works by holding a small write until either:
///   1. The previous segment has been ACKed (round-trip complete), or
///   2. Enough data accumulates to fill a full MSS (~1460 bytes).
///
/// For a **log receiver**, Nagle's algorithm on the *sender side* can introduce up to
/// one RTT of extra latency per small log line, causing "micro-batching" that delays
/// delivery. When combined with TCP delayed ACKs (typically 40–200 ms), this can
/// compound into noticeable latency spikes.
///
/// We disable Nagle via `TCP_NODELAY` (set in `server.rs` on the listening socket)
/// so that each write from the log shipper is sent immediately as its own segment.
/// This trades a small increase in packet count for **lowest possible latency** —
/// critical for real-time log streaming where freshness matters more than bandwidth
/// efficiency. The read side here benefits because data arrives sooner, reducing the
/// time the IO reader spends blocked in `read_buf()` waiting for bytes.
pub async fn run_io_reader(
    mut stream: TcpStream,
    tx: Sender<BytesMut>,
    shutdown: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut buf = BytesMut::with_capacity(BUF_CAPACITY);
    let mut framer = JsonFramer::new();
    let mut total_bytes: u64 = 0;
    let mut total_objects: u64 = 0;

    debug!("IO reader started");

    loop {
        // Stop reading if shutdown was signalled.
        let n = tokio::select! {
            biased;

            _ = shutdown.cancelled() => {
                info!(total_bytes, total_objects, "IO reader stopping due to shutdown");
                return Ok(());
            }

            result = stream.read_buf(&mut buf) => {
                result?
            }
        };

        if n == 0 {
            info!(total_bytes, total_objects, "EOF reached");
            return Ok(());
        }
        total_bytes += n as u64;
        debug!(bytes_read = n, buf_len = buf.len(), depth = framer.depth, "Read from socket");

        // Guard against unbounded growth from a missing closing '}'.
        if buf.len() > MAX_JSON_LEN && framer.depth > 0 {
            warn!(len = buf.len(), depth = framer.depth, "Buffer exceeded max JSON size, discarding");
            buf.clear();
            framer = JsonFramer::new();
            continue;
        }

        let objects = framer.extract_positions(&buf);
        if objects.is_empty() {
            debug!(buf_len = buf.len(), depth = framer.depth, "No complete objects yet");
            continue;
        }
        debug!(count = objects.len(), "Framed complete objects");
        total_objects += objects.len() as u64;

        // Determine how many bytes to drain (up to the end of the last complete object).
        let drain_end = objects.last().unwrap().1;

        // Split off the processed prefix as a single owned chunk so we can split it up cheaply.
        let mut chunk = buf.split_to(drain_end);

        // Shift framer's internal indices because we've removed `drain_end` bytes.
        framer.shift(drain_end);

        // Extract each object from the chunk and send it to the parser.
        let mut consumed = 0usize;
        for &(start, end) in &objects {
            let rel_start = start - consumed;
            if rel_start > 0 {
                // drop leading bytes before the next '{'
                let _ = chunk.split_to(rel_start);
                consumed += rel_start;
            }
            let obj_len = end - start;
            let obj = chunk.split_to(obj_len);
            consumed += obj_len;

            // Send object to parser; await if the parser is backpressuring.
            if tx.send(obj).await.is_err() {
                // Receiver closed — we're done.
                return Ok(());
            }
        }
    }
}
