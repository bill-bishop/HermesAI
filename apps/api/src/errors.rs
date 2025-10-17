
use axum::{http::StatusCode, response::{IntoResponse, Response}};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::Upstream(m) => (StatusCode::BAD_GATEWAY, m),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };
        let body = serde_json::json!({ "error": msg });
        (code, axum::Json(body)).into_response()
    }
}
