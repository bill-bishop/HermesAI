mod routes;
mod models;
mod executor;
mod io;
mod state;
mod config;
mod error;

use axum::{Router};
use tracing_subscriber::EnvFilter;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = config::Config::load()?;
    tracing::info!(?cfg, "config loaded");

    let state = AppState::new();

    let app: Router = routes::app_router(state);

    let addr = std::net::SocketAddr::from(([0,0,0,0], 8080));
    tracing::info!("listening on http://{}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}
