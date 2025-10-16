use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use futures_core::stream::Stream;
use futures_util::stream::{self, StreamExt};
use serde_json::json;
use tokio::sync::broadcast;
use std::time::Duration;

use crate::models::StreamFrame;

/// Replay backlog (seq > from) then live frames from a broadcast receiver.
pub fn ndjson_stream_with_backlog(
    mut backlog: Vec<StreamFrame>,
    mut rx: broadcast::Receiver<StreamFrame>,
    from: u64,
) -> impl IntoResponse {
    // Filter backlog by `from`, sort by seq, and map to NDJSON lines
    backlog.sort_by_key(|f| f.seq);
    let past = backlog
        .into_iter()
        .filter(move |f| f.seq > from)
        .map(|f| Ok::<String, std::convert::Infallible>(serde_json::to_string(&f).unwrap() + "\n"));

    // Live stream: subscribe and forward frames
    let live = stream::unfold(rx, |mut r| async {
        match r.recv().await {
            Ok(f) => Some((Ok::<String, std::convert::Infallible>(serde_json::to_string(&f).unwrap() + "\n"), r)),
            Err(broadcast::error::RecvError::Lagged(_)) => Some((Ok(String::new()), r)), // drop if lagged
            Err(_) => None,
        }
    });

    // Prepend a banner then chain past + live
    let banner = stream::once(async {
        Ok::<String, std::convert::Infallible>(json!({"t":"event","seq":0,"d":"stream-start"}).to_string() + "\n")
    });

    let all = banner.chain(stream::iter(past)).chain(live);
    axum::response::Response::new(axum::body::Body::from_stream(all))
}
