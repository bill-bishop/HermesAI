mod middleware;
mod models;
mod routes;
mod services;
mod state;

use axum::{Router, routing::{get}};
use crate::routes::terminal::{get_terminal, post_terminal};
use crate::routes::file::{get_file, post_file};
use crate::state::SessionManager;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let manager = SessionManager::new();
    
    println!("Starting Agent API...");

    let app = Router::new()
        .route("/terminal", get(get_terminal).post(post_terminal)) // `Result<(), anyhow::Error>` is not a future [E0277]
        .route("/sandbox/{*path}", get(get_file).post(post_file))
        .with_state(manager);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("❌ axum serve error: {e}");
        return Err(Box::new(e).into());
    }

    println!("🚀 agent-api running on http://0.0.0.0:8081");
    
    
    Ok(())
}