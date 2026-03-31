use bytes::BytesMut;
use tokio::sync::{broadcast, mpsc};

use RoggingHub::io_handler::parser::run_parser;

#[tokio::test]
async fn parses_valid_json_and_sends_to_output() {
    let (in_tx, in_rx) = mpsc::channel::<BytesMut>(16);
    let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(16);

    let handle = tokio::spawn(run_parser(in_rx, Some(out_tx), None));

    in_tx.send(BytesMut::from(r#"{"key":"value"}"#)).await.unwrap();
    in_tx.send(BytesMut::from(r#"{"num":42}"#)).await.unwrap();
    drop(in_tx);

    handle.await.unwrap().unwrap();

    let msg1 = out_rx.recv().await.unwrap();
    let msg2 = out_rx.recv().await.unwrap();
    assert!(String::from_utf8_lossy(&msg1).contains("key"));
    assert!(String::from_utf8_lossy(&msg2).contains("42"));
    assert!(out_rx.recv().await.is_none());
}

#[tokio::test]
async fn invalid_json_does_not_send_to_output() {
    let (in_tx, in_rx) = mpsc::channel::<BytesMut>(16);
    let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(16);

    let handle = tokio::spawn(run_parser(in_rx, Some(out_tx), None));

    in_tx.send(BytesMut::from("not json at all")).await.unwrap();
    drop(in_tx);

    handle.await.unwrap().unwrap();
    assert!(out_rx.recv().await.is_none());
}

#[tokio::test]
async fn broadcasts_to_sse() {
    let (in_tx, in_rx) = mpsc::channel::<BytesMut>(16);
    let (sse_tx, mut sse_rx) = broadcast::channel::<bytes::Bytes>(16);

    let handle = tokio::spawn(run_parser(in_rx, None, Some(sse_tx)));

    in_tx.send(BytesMut::from(r#"{"sse":true}"#)).await.unwrap();
    drop(in_tx);

    handle.await.unwrap().unwrap();

    let msg = sse_rx.recv().await.unwrap();
    assert!(String::from_utf8_lossy(&msg).contains("sse"));
}

#[tokio::test]
async fn empty_input_finishes_cleanly() {
    let (_in_tx, in_rx) = mpsc::channel::<BytesMut>(16);
    drop(_in_tx);

    let result = run_parser(in_rx, None, None).await;
    assert!(result.is_ok());
}
