use axum::{
    extract::{Path, Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::info;

use crate::services::session_manager::SessionManager;
use crate::middleware::auth::AuthHeader;

#[derive(Deserialize)]
pub struct FileWriteBody {
    content: String,
}

pub async fn get_file(
    State(manager): State<SessionManager>,
    auth: AuthHeader,
    Path(path): Path<String>,
) -> impl IntoResponse {
    match manager.get_file(auth.token(), &path).await {
        Ok(content) => (StatusCode::OK, content),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {e}\n")),
    }
}

pub async fn post_file(
    State(manager): State<SessionManager>,
    auth: AuthHeader,
    Path(path): Path<String>,
    Json(body): Json<FileWriteBody>,
) -> impl IntoResponse {
    info!("POST /sandbox/{path} token='{}'", auth.token());
    match manager.write_file(auth.token(), &path, &body.content).await {
        Ok(msg) => (StatusCode::OK, msg),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {e}\n")),
    }
}
