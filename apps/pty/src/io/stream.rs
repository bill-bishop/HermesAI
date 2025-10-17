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
    backlog.sort_by_key(|f| f.seq);

    // Past frames (resume support)
    let past = backlog
        .into_iter()
        .filter(move |f| f.seq > from)
        .map(|f| Ok::<String, std::convert::Infallible>(serde_json::to_string(&f).unwrap() + "\n"));

    // Live frames: stop the stream after emitting an `exit:*` event
    let live = stream::unfold((rx, false), |(mut r, done)| async move {
        if done {
            return None;
        }
        match r.recv().await {
            Ok(f) => {
                let is_exit = f.t == "event" && f.d.starts_with("exit:");
                let line = serde_json::to_string(&f).unwrap() + "\n";
                Some((Ok::<String, std::convert::Infallible>(line), (r, is_exit)))
            }
            // If we lagged, keep going; emit nothing
            Err(broadcast::error::RecvError::Lagged(_)) => Some((Ok(String::new()), (r, false))),
            // Channel closed → end stream
            Err(_) => None,
        }
    });

    // Banner then backlog then live
    let banner = stream::once(async {
        Ok::<String, std::convert::Infallible>(
            json!({"t":"event","seq":0,"d":"stream-start"}).to_string() + "\n",
        )
    });

    let all = banner.chain(stream::iter(past)).chain(live);
    axum::response::Response::new(axum::body::Body::from_stream(all))
}
