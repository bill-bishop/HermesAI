
use axum::{Router, routing::{get, post}};
use crate::{handlers::{health, terminal}, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/version", get(health::version))
        .route("/terminal", post(terminal::post_terminal).get(terminal::get_terminal))
        .route("/terminal/signal", post(terminal::post_signal))
        .with_state(state)
}
