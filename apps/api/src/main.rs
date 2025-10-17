
mod config; mod routes; mod telemetry; mod state; mod errors; mod middleware; mod handlers; mod services; mod models; mod util; mod cache;
use axum::Router;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init();
    let cfg = config::Config::from_env()?;
    let state = state::AppState::build(cfg.clone()).await?;
    let app: Router = routes::router(state.clone());
    let addr = cfg.bind;
    info!(%addr, "agent-api listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
