use axum::response::IntoResponse;
use futures_util::stream::{self, StreamExt};
use tokio::sync::broadcast;
use bytes::Bytes;
use serde_json::json;
use crate::models::StreamFrame;

pub fn ndjson_stream_with_backlog(
    mut backlog: Vec<StreamFrame>,
    rx: broadcast::Receiver<StreamFrame>,
    from: u64,
) -> impl IntoResponse {
    backlog.sort_by_key(|f| f.seq);

    let banner = stream::once(async {
        let s = json!({"t":"event","seq":0,"d":"stream-start"}).to_string() + "\n";
        Ok::<Bytes, std::convert::Infallible>(Bytes::from(s))
    });

    let past = backlog
        .into_iter()
        .filter(move |f| f.seq > from)
        .map(|f| {
            let line = serde_json::to_string(&f).unwrap() + "\n";
            Ok::<Bytes, std::convert::Infallible>(Bytes::from(line))
        });

    let live = stream::unfold((rx, false), |(mut r, done)| async move {
        if done { return None; }
        match r.recv().await {
            Ok(f) => {
                let is_exit = f.t == "event" && f.d.starts_with("exit:");
                let line = serde_json::to_string(&f).unwrap() + "\n";
                tokio::task::yield_now().await; // nudge hyper to flush
                Some((Ok::<Bytes, std::convert::Infallible>(Bytes::from(line)), (r, is_exit)))
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                tokio::task::yield_now().await;
                Some((Ok(Bytes::new()), (r, false)))
            }
            Err(_) => None,
        }
    });

    let all = banner.chain(stream::iter(past)).chain(live);

    axum::response::Response::builder()
        .status(200)
        .header("content-type", "application/x-ndjson")
        .header("transfer-encoding", "chunked")
        .body(axum::body::Body::from_stream(all))
        .unwrap()
}
