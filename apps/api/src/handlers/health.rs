
use axum::Json;

pub async fn health() -> &'static str { "ok" }
pub async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "name":"agent-api", "version":"0.1.0" }))
}
