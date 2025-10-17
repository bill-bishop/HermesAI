use axum::{extract::{State}, Json};
use crate::{middleware::auth::AuthHeader, models::terminal::{TerminalBody, TerminalResponse}, state::SessionManager};
use tracing::info;

pub async fn post_terminal(
    State(manager): State<SessionManager>,
    auth: AuthHeader,
    Json(body): Json<TerminalBody>
) -> Result<Json<TerminalResponse>, (axum::http::StatusCode, String)> {
    let token = auth.token();
    let cmd = body.cmd;
    info!("POST /terminal cmd='{}' token='{}'", cmd, token);
    let frames = manager.write_and_read(&token, cmd).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(TerminalResponse { running: true, frames, message: "ok".into() }))
}

pub async fn get_terminal(
    State(manager): State<SessionManager>,
    auth: AuthHeader
) -> Result<Json<TerminalResponse>, (axum::http::StatusCode, String)> {
    let token = auth.token();
    let frames = manager.read_latest(&token).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(TerminalResponse { running: true, frames, message: "ok".into() }))
}
