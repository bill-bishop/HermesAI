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
    .with_state(state)
}

async fn start_session(State(state):State<AppState>,Json(req):Json<SessionRequest>)->Result<Json<SessionResponse>,(StatusCode,String)>{
    let id=ids::new_id("s");
    let h=pty::spawn_pty_shell(req.shell.as_deref().unwrap_or("/bin/bash"),req.cols.unwrap_or(120),req.rows.unwrap_or(32)).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))?;
    state.sessions.write().await.insert(id.clone(),h);
    Ok(Json(SessionResponse{session_id:id.clone(),stream_url:format!("/sessions/{}/stream?from=0",id),write_url:String::new(),resize_url:String::new(),close_url:String::new()}))
}

async fn stream_session(State(state):State<AppState>,Path(id):Path<String>)->Result<impl axum::response::IntoResponse,(StatusCode,String)>{
    let guard=state.sessions.read().await;
    let Some(h)=guard.get(&id) else{return Err((StatusCode::NOT_FOUND,"session not found".into()));};
    let rx=h.tx.subscribe();
    Ok(crate::io::stream::ndjson_stream(rx,0))
}
