
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Debug)]
pub struct NodeSessionResponse {
    pub session_id: String,
    pub stream_url: String,
    pub write_url: String,
    pub resize_url: String,
    pub close_url: String,
}

#[derive(Serialize)]
pub struct NodeSessionRequest {
    pub mode: String,          // "interactive"
    pub profile: String,       // "bash"
    pub cols: u16,
    pub rows: u16,
}

#[derive(Serialize)]
pub struct NodeWriteRequest { pub data: String }

#[derive(Serialize)]
pub struct NodeSignalRequest { pub signal: String }

// For streaming, we reuse StreamFrame from common.
