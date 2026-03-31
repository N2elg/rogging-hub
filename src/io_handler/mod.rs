pub mod framer;
pub mod io;
pub mod parser;

use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use bytes::BytesMut;
use tracing::{debug, info, warn};

type ErrBox = Box<dyn std::error::Error + Send + Sync + 'static>;

pub async fn handle_json_stream(
    stream: TcpStream,
    output_tx: Option<mpsc::Sender<Vec<u8>>>,
    sse_tx: Option<tokio::sync::broadcast::Sender<bytes::Bytes>>,
    parser_rt: Arc<tokio::runtime::Runtime>,
    shutdown: CancellationToken,
) -> Result<(), ErrBox> {
    let peer = stream.peer_addr().ok();
    info!(?peer, "Connection accepted, spawning IO reader + parser");

    let (tx, rx) = mpsc::channel::<BytesMut>(1024);

    // Spawn parser on the dedicated parser thread pool.
    let parser_handle = parser_rt.spawn(async move {
        parser::run_parser(rx, output_tx, sse_tx).await
    });

    // IO reader runs on the current (IO) runtime.
    // It will stop on EOF or when the shutdown token is cancelled.
    let io_result = io::run_io_reader(stream, tx, shutdown).await;
    debug!(?peer, "IO reader finished");

    match parser_handle.await {
        Ok(parser_res) => {
            if let Err(ref e) = parser_res {
                warn!(?peer, error = %e, "Parser task returned error");
            }
            if let Err(ref e) = io_result {
                warn!(?peer, error = %e, "IO reader returned error");
            }
            parser_res?;
            io_result?;
            info!(?peer, "Connection closed cleanly");
            Ok(())
        }
        Err(join_err) => {
            warn!(?peer, error = %join_err, "Parser task panicked");
            Err(Box::new(join_err))
        }
    }
}
