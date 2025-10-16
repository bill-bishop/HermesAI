use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use crate::models::*;
use crate::state::{ids, AppState};
use crate::executor::{pty, spawn};
use axum::http::StatusCode;
use serde::Deserialize;

pub fn app_router(state: AppState) -> Router {
    Router::new()
        // health
        .route("/health", get(|| async { "ok" }))
        // non-PTY exec
        .route("/exec", post(exec))
        .route("/stream/:id", get(stream_job))
        .route("/status/:id", get(status_job))
        // PTY
        .route("/sessions", post(start_session))
        .route("/sessions/:id/stream", get(stream_session))
        .route("/sessions/:id/write", post(write_session))
        .route("/sessions/:id/resize", post(resize_session))
        .route("/sessions/:id/close", post(close_session))
        .with_state(state)
}

#[derive(Deserialize)]
struct FromParam {
    from: Option<u64>,
}

// ---------- PTY ----------
async fn start_session(
    State(state): State<AppState>,
    Json(req): Json<SessionRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, String)> {
    let id = ids::new_id("s");
    let shell = req.shell.as_deref().unwrap_or("/bin/bash");
    let h = pty::spawn_pty_shell(shell, req.cols.unwrap_or(120), req.rows.unwrap_or(32))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state.sessions.write().await.insert(id.clone(), h);
    Ok(Json(SessionResponse {
        session_id: id.clone(),
        stream_url: format!("/sessions/{}/stream?from=0", id),
        write_url: format!("/sessions/{}/write", id),
        resize_url: format!("/sessions/{}/resize", id),
        close_url: format!("/sessions/{}/close", id),
    }))
}

async fn stream_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<FromParam>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    let guard = state.sessions.read().await;
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "session not found".into())); };
    let rx = h.tx.subscribe();
    let backlog = {
        let b = h.backlog.lock();
        b.iter().cloned().collect::<Vec<_>>()
    };
    Ok(crate::io::stream::ndjson_stream_with_backlog(backlog, rx, q.from.unwrap_or(0)))
}


async fn write_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<WriteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let guard = state.sessions.read().await;
    let Some(h) = guard.get(&id) else {
        return Err((StatusCode::NOT_FOUND, "session not found".into()));
    };
    tracing::debug!("write_session: {} bytes to {}", body.data.len(), id);
    pty::write_pty(h, &body.data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn resize_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ResizeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let guard = state.sessions.read().await;
    let Some(h) = guard.get(&id) else {
        return Err((StatusCode::NOT_FOUND, "session not found".into()));
    };
    pty::resize_pty(h, body.cols, body.rows)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn close_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let guard = state.sessions.read().await;
    let Some(h) = guard.get(&id) else {
        return Err((StatusCode::NOT_FOUND, "session not found".into()));
    };
    pty::close_pty(h)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ---------- non-PTY exec ----------
async fn exec(
    State(state): State<AppState>,
    Json(req): Json<ExecRequest>,
) -> Result<Json<ExecResponse>, (StatusCode, String)> {
    if req.cmd.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "cmd required".into()));
    }
    let id = ids::new_id("j");
    let handle = spawn::spawn_noninteractive(req.cmd.clone(), req.cwd.clone()).await;
    state.jobs.write().await.insert(id.clone(), handle);
    Ok(Json(ExecResponse {
        job_id: id.clone(),
        stream_url: format!("/stream/{}?from=0", id),
        status_url: format!("/status/{}", id),
        cancel_url: String::new(),
    }))
}

async fn stream_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<FromParam>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    let jobs = state.jobs.read().await;
    let Some(h) = jobs.get(&id) else { return Err((StatusCode::NOT_FOUND, "job not found".into())); };
    let rx = h.tx.subscribe();
    let backlog = {
        let b = h.backlog.lock();
        b.iter().cloned().collect::<Vec<_>>()
    };
    Ok(crate::io::stream::ndjson_stream_with_backlog(backlog, rx, q.from.unwrap_or(0)))
}


async fn status_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
     // Scope the read guard so it drops before we build the JSON (avoids borrow issues)
     let (state_str, exit_code, seq_latest) = {
         let jobs = state.jobs.read().await;
         let Some(h) = jobs.get(&id) else {
             return Err((StatusCode::NOT_FOUND, "job not found".into()));
         };
         let state_str = if h.exit_code.lock().is_some() {
             "exited".to_string()
         } else {
             "running".to_string()
         };
         let exit_code = *h.exit_code.lock();
         let seq_latest = *h.latest_seq.lock();
         (state_str, exit_code, seq_latest)
     };
     Ok(Json(StatusResponse { state: state_str, exit_code, seq_latest }))
}
