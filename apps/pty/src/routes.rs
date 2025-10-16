use axum::{extract::{Path, Query, State}, routing::{get, post}, Json, Router};
use serde::Deserialize;
use crate::models::*;
use crate::state::{AppState, ids, SessionHandle};
use crate::executor::{spawn, pty};
use axum::http::StatusCode;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct FromParam { pub from: Option<u64> }

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/exec", post(exec))
        .route("/stream/:id", get(stream_job))
        .route("/status/:id", get(status_job))
        .route("/cancel/:id", post(cancel_job))
        .route("/sessions", post(start_session))
        .route("/sessions/:id/stream", get(stream_session))
        .route("/sessions/:id/write", post(write_session))
        .route("/sessions/:id/resize", post(resize_session))
        .route("/sessions/:id/close", post(close_session))
        .with_state(state)
}

async fn exec(State(state): State<AppState>, Json(req): Json<ExecRequest>) -> Result<Json<ExecResponse>, (StatusCode, String)> {
    if req.cmd.is_empty() { return Err((StatusCode::BAD_REQUEST, "cmd required".into())); }
    let id = ids::new_id("j");
    let handle = spawn::spawn_noninteractive(req.cmd.clone(), req.cwd.clone()).await;
    state.jobs.write().insert(id.clone(), handle);
    let resp = ExecResponse {
        job_id: id.clone(),
        stream_url: format!("/stream/{}?from=0", id),
        status_url: format!("/status/{}", id),
        cancel_url: format!("/cancel/{}", id),
    };
    Ok(Json(resp))
}

async fn stream_job(State(state): State<AppState>, Path(id): Path<String>, Query(q): Query<FromParam>) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    let guard = state.jobs.read();
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "job not found".into())); };
    let rx = h.tx.subscribe();
    let from = q.from.unwrap_or(0);
    Ok(crate::io::stream::ndjson_stream(rx, from))
}

async fn status_job(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    let guard = state.jobs.read();
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "job not found".into())); };
    let latest = *h.latest_seq.lock();
    let exit = *h.exit_code.lock();
    let state_s = if exit.is_some() { "exited" } else { "running" };
    Ok(Json(StatusResponse { state: state_s.into(), exit_code: exit, seq_latest: latest }))
}

async fn cancel_job(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // TODO: wire kill handle
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn start_session(State(state): State<AppState>, Json(req): Json<SessionRequest>) -> Result<Json<SessionResponse>, (StatusCode, String)> {
    let id = ids::new_id("s");
    let shell = req.shell.clone().unwrap_or("/bin/bash".into());
    let cols = req.cols.unwrap_or(120);
    let rows = req.rows.unwrap_or(32);
    let h = pty::spawn_pty_shell(&shell, cols, rows, req.cwd.clone(), req.env.clone())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("spawn pty: {}", e)))?;
    state.sessions.write().insert(id.clone(), h);
    Ok(Json(SessionResponse {
        session_id: id.clone(),
        stream_url: format!("/sessions/{}/stream?from=0", id),
        write_url: format!("/sessions/{}/write", id),
        resize_url: format!("/sessions/{}/resize", id),
        close_url: format!("/sessions/{}/close", id),
    }))
}

async fn stream_session(State(state): State<AppState>, Path(id): Path<String>, Query(q): Query<FromParam>) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    let guard = state.sessions.read();
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "session not found".into())); };
    let rx = h.tx.subscribe();
    let from = q.from.unwrap_or(0);
    Ok(crate::io::stream::ndjson_stream(rx, from))
}

async fn write_session(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<WriteRequest>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let guard = state.sessions.read();
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "session not found".into())); };
    crate::executor::pty::write_pty(h, &body.data).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn resize_session(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<ResizeRequest>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let guard = state.sessions.read();
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "session not found".into())); };
    crate::executor::pty::resize_pty(h, body.cols, body.rows).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn close_session(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut guard = state.sessions.write();
    let Some(h) = guard.remove(&id) else { return Err((StatusCode::NOT_FOUND, "session not found".into())); };
    crate::executor::pty::close_pty(&h).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({"ok": true})))
}
