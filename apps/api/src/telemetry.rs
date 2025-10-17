
use tracing_subscriber::{EnvFilter, fmt};

pub fn init() {
    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "agent_api=debug,axum=info,hyper=info,reqwest=info".into());
    fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .compact()
        .init();
}
