use axum::{extract::{Path, State, Query}, routing::{get, post}, Json, Router};
use axum::http::StatusCode;
use tower_http::services::ServeDir;
use serde::Deserialize;
use tokio::fs;
use crate::models::*;
use crate::state::{AppState, ids};
use crate::executor::{pty, spawn};

pub fn app_router(state: AppState) -> Router {
    // Serve static files from /sandbox/preview on the host
    let preview_service = ServeDir::new("/sandbox/preview");
    
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/sandbox/*path", get(get_file))
        .route("/sandbox/*path", post(put_file))
        .route("/exec", post(exec))
        .route("/sessions", post(start_session))
        .route("/sessions/:id/stream", get(stream_session))
        .route("/sessions/:id/write", post(write_session))
        .route("/sessions/:id/resize", post(resize_session))
        .route("/sessions/:id/close", post(close_session))
        .route("/stream/:id", get(stream_job))
        .route("/stream/:id/close", post(close_job_stream))
        .route("/status/:id", get(status_job))
        .nest_service("/preview", preview_service) // 👈 serve static files here
        .with_state(state)
}

#[derive(Deserialize)]
struct FromParam { from: Option<u64> }

#[derive(Deserialize)]
struct FileWriteBody {
    content: String,
}

// GET /files/:path
async fn get_file(
    Path(path): Path<String>,
) -> Result<String, (StatusCode, String)> {
    let full_path = format!("/sandbox/{}", path); // adjust base dir if needed
    match fs::read_to_string(&full_path).await {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err((StatusCode::NOT_FOUND, format!("file not found: {full_path}")))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// POST /files/:path
async fn put_file(
    Path(path): Path<String>,
    Json(body): Json<FileWriteBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let full_path = format!("/sandbox/{}", path);
    if let Some(parent) = std::path::Path::new(&full_path).parent() {
        if let Err(e) = fs::create_dir_all(parent).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    }

    match fs::write(&full_path, body.content.as_bytes()).await {
        Ok(_) => Ok(Json(serde_json::json!({ "ok": true, "path": full_path }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}


async fn start_session(
    State(state): State<AppState>,
    Json(req): Json<SessionRequest>
) -> Result<Json<SessionResponse>, (StatusCode, String)> {
    if req.mode.as_str() != "interactive" {
        return Err((StatusCode::BAD_REQUEST, "only interactive mode supported".into()));
    }
    let id = ids::new_id("s");
    let cols = req.cols.unwrap_or(120);
    let rows = req.rows.unwrap_or(32);
    let h = pty::spawn_pty_shell(req.profile.clone(), cols, rows)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state.sessions.write().await.insert(id.clone(), h);

    Ok(Json(SessionResponse {
        session_id: id.clone(),
        stream_url: format!("/sessions/{}/stream?from=0", id),
        write_url:  format!("/sessions/{}/write", id),
        resize_url: format!("/sessions/{}/resize", id),
        close_url:  format!("/sessions/{}/close", id),
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
    Json(body): Json<WriteRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let guard = state.sessions.read().await;
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "session not found".into())); };
    tracing::debug!("write_session: {} bytes to {}", body.data.len(), id);
    pty::write_pty(h, &body.data).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn resize_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ResizeRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let guard = state.sessions.read().await;
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "session not found".into())); };
    pty::resize_pty(h, body.cols, body.rows).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn close_session(
    State(state): State<AppState>,
    Path(id): Path<String>
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let guard = state.sessions.read().await;
    let Some(h) = guard.get(&id) else { return Err((StatusCode::NOT_FOUND, "session not found".into())); };
    pty::close_pty(h).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn exec(
    State(state): State<AppState>,
    Json(req): Json<ExecRequest>
) -> Result<Json<ExecResponse>, (StatusCode, String)> {
    if req.cmd.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "cmd required".into()));
    }
    let id = ids::new_id("j");
    let handle = spawn::spawn_noninteractive(req.cmd.clone(), req.cwd.clone()).await;
    state.jobs.write().await.insert(id.clone(), handle);
    Ok(Json(ExecResponse {
        job_id: id.clone(),
        stream_url: format!("/stream/{id}?from=0"),
        status_url: format!("/status/{id}"),
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

/// POST /stream/:id/close — gracefully close a running job stream
async fn close_job_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use tracing::info;

    let mut jobs = state.jobs.write().await;
    if let Some(mut handle) = jobs.remove(&id) {
        info!("Closing job {}", id);

        // Try to kill the process
        if let Ok(mut child) = handle.child.lock().await.try_wait() {
            if child.is_none() {
                info!("Job {} already exited", id);
            } else {
                if let Err(e) = handle.child.lock().await.kill().await {
                    info!("Job {} kill failed: {}", id, e);
                } else {
                    info!("Job {} terminated successfully", id);
                }
            }
        }

        Ok(Json(serde_json::json!({ "ok": true, "closed": id })))
    } else {
        Err((StatusCode::NOT_FOUND, format!("job {id} not found")))
    }
}


async fn status_job(
    State(state): State<AppState>,
    Path(id): Path<String>
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    let (state_str, exit_code, seq_latest) = {
        let jobs = state.jobs.read().await;
        let Some(h) = jobs.get(&id) else { return Err((StatusCode::NOT_FOUND, "job not found".into())); };
        let exit_code = *h.exit_code.lock();
        let seq_latest = h.latest_seq.load(std::sync::atomic::Ordering::Relaxed);
        let state_str = if exit_code.is_some() { "exited".into() } else { "running".into() };
        (state_str, exit_code, seq_latest)
    };
    Ok(Json(StatusResponse { state: state_str, exit_code, seq_latest }))
}
