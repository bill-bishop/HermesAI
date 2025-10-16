use axum::{routing::get, Router};
use tracing_subscriber::{fmt, EnvFilter};

mod models;
mod state;
mod executor;
mod io;
mod config;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // Build state and router
    let state = state::AppState::new();
    let app = routes::app_router(state);

    // Bind listener (use same port as before)
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);

    // Serve via axum::serve (Axum 0.7+)
    axum::serve(listener, app).await?;

    Ok(())
}
