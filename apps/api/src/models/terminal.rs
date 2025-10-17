use axum_macros::FromRequest;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(FromRequest, Deserialize)]
pub struct TerminalBody {
    #[from_request(via(Json))]
    pub cmd: String,
}

#[derive(Serialize)]
pub struct TerminalResponse {
    pub running: bool,
    pub frames: Vec<String>,
    pub message: String,
}
