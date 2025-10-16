use axum::{extract::{Path,State},routing::{get,post},Json,Router};
use crate::models::*;
use crate::state::{AppState,ids};
use crate::executor::pty;
use axum::http::StatusCode;

pub fn app_router(state:AppState)->Router{
    Router::new()
    .route("/health",get(||async{"ok"}))
    .route("/sessions",post(start_session))
    .route("/sessions/:id/stream",get(stream_session))
    .route("/sessions/:id/write",post(write_session))
    .route("/sessions/:id/resize",post(resize_session))
    .route("/sessions/:id/close",post(close_session))
    .with_state(state)
}

async fn start_session(State(state):State<AppState>,Json(req):Json<SessionRequest>)->Result<Json<SessionResponse>,(StatusCode,String)>{
    let id=ids::new_id("s");
    let shell=req.shell.as_deref().unwrap_or("/bin/bash");
    let h=pty::spawn_pty_shell(shell,req.cols.unwrap_or(120),req.rows.unwrap_or(32)).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))?;
    state.sessions.write().await.insert(id.clone(),h);
    Ok(Json(SessionResponse{
        session_id:id.clone(),
        stream_url:format!("/sessions/{}/stream?from=0",id),
        write_url:format!("/sessions/{}/write",id),
        resize_url:format!("/sessions/{}/resize",id),
        close_url:format!("/sessions/{}/close",id),
    }))
}

async fn stream_session(State(state):State<AppState>,Path(id):Path<String>)->Result<impl axum::response::IntoResponse,(StatusCode,String)>{
    let guard=state.sessions.read().await;
    let Some(h)=guard.get(&id) else{return Err((StatusCode::NOT_FOUND,"session not found".into()));};
    let rx=h.tx.subscribe();
    Ok(crate::io::stream::ndjson_stream(rx,0))
}

async fn write_session(State(state):State<AppState>,Path(id):Path<String>,Json(body):Json<WriteRequest>)->Result<Json<serde_json::Value>,(StatusCode,String)>{
    let guard=state.sessions.read().await;
    let Some(h)=guard.get(&id) else{return Err((StatusCode::NOT_FOUND,"session not found".into()));};
    pty::write_pty(h,&body.data).await.map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))?;
    Ok(Json(serde_json::json!({"ok":true})))
}

async fn resize_session(State(state):State<AppState>,Path(id):Path<String>,Json(body):Json<ResizeRequest>)->Result<Json<serde_json::Value>,(StatusCode,String)>{
    let guard=state.sessions.read().await;
    let Some(h)=guard.get(&id) else{return Err((StatusCode::NOT_FOUND,"session not found".into()));};
    pty::resize_pty(h,body.cols,body.rows).await.map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))?;
    Ok(Json(serde_json::json!({"ok":true})))
}

async fn close_session(State(state):State<AppState>,Path(id):Path<String>)->Result<Json<serde_json::Value>,(StatusCode,String)>{
    let guard=state.sessions.read().await;
    let Some(h)=guard.get(&id) else{return Err((StatusCode::NOT_FOUND,"session not found".into()));};
    pty::close_pty(h).await.map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))?;
    Ok(Json(serde_json::json!({"ok":true})))
}
