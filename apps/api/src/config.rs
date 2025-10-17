
use std::net::SocketAddr;

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub node_secret: String,
    pub default_cols: u16,
    pub default_rows: u16,
    pub longpoll_ms: u64,
    pub tail_size: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let bind = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
        Ok(Self {
            bind: bind.parse()?,
            node_secret: std::env::var("NODE_SECRET").unwrap_or_default(),
            default_cols: std::env::var("DEFAULT_COLS").ok().and_then(|v| v.parse().ok()).unwrap_or(120),
            default_rows: std::env::var("DEFAULT_ROWS").ok().and_then(|v| v.parse().ok()).unwrap_or(32),
            longpoll_ms: std::env::var("LONGPOLL_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(10_000),
            tail_size: std::env::var("TAIL_SIZE").ok().and_then(|v| v.parse().ok()).unwrap_or(30),
        })
    }
}
