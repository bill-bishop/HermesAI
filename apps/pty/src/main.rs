mod routes;
mod models;
mod executor;
mod io;
mod state;
mod config;
mod error;

use tracing_subscriber::EnvFilter;
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let cfg = config::Config::load()?;
    tracing::info!(?cfg, "config loaded");

    let state = AppState::new();
    let app = routes::app_router(state);
    let addr = std::net::SocketAddr::from(([0,0,0,0], 8080));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
