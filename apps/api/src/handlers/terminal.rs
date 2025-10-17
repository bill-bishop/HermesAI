
use axum::{extract::State, Json};
use axum::http::StatusCode;
use crate::{state::AppState, middleware::auth::AgentContext, models::terminal::*};

pub async fn post_terminal(
    State(app): State<AppState>,
    agent: AgentContext,
    Json(req): Json<PostTerminalRequest>,
) -> Result<Json<PostTerminalResponse>, (StatusCode, String)> {
    let result = app.session_mgr.write_then_read(&agent, req.cmd, app.cfg.longpoll_ms).await
        .map_err(internal)?;
    Ok(Json(result))
}

pub async fn get_terminal(
    State(app): State<AppState>,
    agent: AgentContext,
) -> Result<Json<GetTerminalResponse>, (StatusCode, String)> {
    let result = app.session_mgr.read_new_or_tail(&agent, app.cfg.longpoll_ms, app.cfg.tail_size).await
        .map_err(internal)?;
    Ok(Json(result))
}

fn internal(e: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

pub async fn post_signal(
    State(app): State<AppState>,
    agent: AgentContext,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let sig = body.get("signal").and_then(|v| v.as_str()).unwrap_or("INT").to_string();
    app.session_mgr.signal(&agent, sig).await.map_err(internal)?;
    Ok(Json(serde_json::json!({"ok": true})))
}
