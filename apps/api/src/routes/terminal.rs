use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::info;

use crate::services::session_manager::SessionManager;
use crate::middleware::auth::AuthHeader;
use crate::models::terminal::TerminalBody;

pub async fn post_terminal(
    State(manager): State<SessionManager>,
    auth: AuthHeader,
    Json(body): Json<TerminalBody>,
) -> impl IntoResponse {
    info!("POST /terminal cmd='{}' token='{}'", body.cmd, auth.token());
    match manager.execute(auth.token(), &body.cmd).await {
        Ok(output) => (StatusCode::OK, output),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {e}\n")),
    }
}

pub async fn get_terminal(
    State(manager): State<SessionManager>,
    auth: AuthHeader,
) -> impl IntoResponse {
    match manager.get_terminal_tail(auth.token()).await {
        Ok(tail) => (StatusCode::OK, tail),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {e}\n")),
    }
}
// UPDATE
