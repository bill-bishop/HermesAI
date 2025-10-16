use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum::body::{Body, Bytes};
use futures::{Stream, StreamExt};
use serde_json::json;
use tokio::sync::broadcast;
use crate::models::StreamFrame;

pub fn ndjson_stream(mut rx: broadcast::Receiver<StreamFrame>, from: u64) -> impl IntoResponse {
    // Best-effort: we just start from new messages. Clients should reconnect quickly.
    let s = async_stream::stream! {
        // Initial event to indicate start
        let banner = json!({"t":"event","seq":from,"d":"stream-start"}).to_string() + "\n";
        yield Ok::<Bytes, std::io::Error>(Bytes::from(banner));
        loop {
            match rx.recv().await {
                Ok(frame) => {
                    let line = serde_json::to_string(&frame).unwrap() + "\n";
                    yield Ok::<Bytes, std::io::Error>(Bytes::from(line));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    let body = Body::from_stream(s);
    (StatusCode::OK, body)
}
