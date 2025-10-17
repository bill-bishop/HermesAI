
use crate::{config::Config, services::node_client::NodeClient};
use crate::cache::{Cache, MemoryCache};
use std::sync::Arc;
use dashmap::DashMap;
use crate::middleware::auth::AgentContext;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub node: NodeClient,
    pub cache: Arc<dyn Cache + Send + Sync>,
    pub session_mgr: SessionManager,
}

#[derive(Clone)]
pub struct SessionManager {
    pub cfg: Config,
    pub node: NodeClient,
    pub cache: Arc<dyn Cache + Send + Sync>,
    pub sessions: Arc<DashMap<String, String>>, // ws_key -> node_session_id
}

impl AppState {
    pub async fn build(cfg: Config) -> anyhow::Result<Self> {
        let node = NodeClient::new(cfg.node_secret.clone());
        let cache: Arc<dyn Cache + Send + Sync> = Arc::new(MemoryCache::new());
        let sessions = Arc::new(DashMap::new());
        let session_mgr = SessionManager {
            cfg: cfg.clone(),
            node: node.clone(),
            cache: cache.clone(),
            sessions
        };
        Ok(Self { cfg, node, cache, session_mgr })
    }
}

impl SessionManager {
    // workspace-scoped cache key (user + workspace)
    pub fn ws_key(&self, user_id: &str, ws_id: &str) -> String {
        format!("{}::{}", user_id, ws_id)
    }
}

// Hardcoded token map for MVP: edit here
pub fn resolve_token(token: &str) -> Option<AgentContext> {
    // Example: set via env or inline
    // token "tok_me" maps to your node and workspace
    let mapping = vec![
        ("tok_me", ("user_me", "ws_me", std::env::var("NODE_URL_ME").unwrap_or_else(|_| "http://pty-will:8080".into()))),
        ("tok_coo", ("user_coo", "ws_coo", std::env::var("NODE_URL_COO").unwrap_or_else(|_| "http://pty-grace:8080".into()))),
    ];
    for (t,(u,w,n)) in mapping {
        if token == t { return Some(AgentContext { user_id: u.into(), workspace_id: w.into(), node_url: n.into() }); }
    }
    None
}
