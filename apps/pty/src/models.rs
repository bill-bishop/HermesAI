use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ExecRequest {
    pub cmd: Vec<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ExecResponse {
    pub job_id: String,
    pub stream_url: String,
    pub status_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionRequest {
    pub shell: Option<String>,
    pub login: Option<bool>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub stream_url: String,
    pub write_url: String,
    pub resize_url: String,
    pub close_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamFrame {
    pub t: String,        // "stdout" | "stderr" | "event"
    pub seq: u64,
    pub d: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub state: String, // queued | running | exited
    pub exit_code: Option<i32>,
    pub seq_latest: u64,
}

#[derive(Debug, Deserialize)]
pub struct WriteRequest {
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct ResizeRequest {
    pub cols: u16,
    pub rows: u16,
}
