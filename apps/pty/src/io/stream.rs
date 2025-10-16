use axum::response::IntoResponse;
use futures_util::stream::{self, StreamExt};
use tokio::sync::broadcast;

use crate::models::StreamFrame;
use serde_json::json;

pub fn ndjson_stream_with_backlog(
    mut backlog: Vec<StreamFrame>,
    rx: broadcast::Receiver<StreamFrame>,
    from: u64,
) -> impl IntoResponse {
    let mut backlog = backlog;
    backlog.sort_by_key(|f| f.seq);
    let past = backlog
        .into_iter()
        .filter(move |f| f.seq > from)
        .map(|f| Ok::<String, std::convert::Infallible>(serde_json::to_string(&f).unwrap() + "\n"));

    let live = stream::unfold(rx, |mut r| async {
        match r.recv().await {
            Ok(f) => Some((Ok::<String, std::convert::Infallible>(serde_json::to_string(&f).unwrap() + "\n"), r)),
            Err(broadcast::error::RecvError::Lagged(_)) => Some((Ok(String::new()), r)),
            Err(_) => None,
        }
    });

    let banner = stream::once(async {
        Ok::<String, std::convert::Infallible>(json!({"t":"event","seq":0,"d":"stream-start"}).to_string() + "\n")
    });

    let all = banner.chain(stream::iter(past)).chain(live);
    axum::response::Response::new(axum::body::Body::from_stream(all))
}
