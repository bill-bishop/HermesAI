
use serde::{Serialize, Deserialize};
use super::common::StreamFrame;

#[derive(Deserialize)]
pub struct PostTerminalRequest { pub cmd: String }

#[derive(Serialize)]
pub struct PostTerminalResponse {
    pub created: bool,
    pub running: bool,
    pub frames: Vec<StreamFrame>,
    pub next_from: u64,
    pub advice: Option<String>,
}

#[derive(Serialize)]
pub struct GetTerminalResponse {
    pub running: bool,
    pub frames: Vec<StreamFrame>,
    pub tail: Option<Vec<StreamFrame>>,
    pub message: Option<String>,
    pub next_from: u64,
}
