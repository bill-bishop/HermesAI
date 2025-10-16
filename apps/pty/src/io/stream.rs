use axum::response::IntoResponse;
use axum::body::{Body, Bytes};
use serde_json::json;
use tokio::sync::broadcast;
use crate::models::StreamFrame;
pub fn ndjson_stream(mut rx:broadcast::Receiver<StreamFrame>, from:u64)->impl IntoResponse{
    let s=async_stream::stream!{
        let banner=json!({"t":"event","seq":from,"d":"stream-start"}).to_string()+"\n";
        yield Ok::<Bytes,std::io::Error>(Bytes::from(banner));
        while let Ok(frame)=rx.recv().await{let line=serde_json::to_string(&frame).unwrap()+"\n";yield Ok(Bytes::from(line));}
    };
    (axum::http::StatusCode::OK,Body::from_stream(s))
}
