use serde::{Deserialize, Serialize};

// we're using axum's macros 
#[derive(Deserialize, Debug)]
pub struct TerminalBody {
    pub cmd: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ExecResponse {
    pub job_id: String,
    pub stream_url: Option<String>,
    pub status_url: Option<String>,
    pub cancel_url: Option<String>,
}