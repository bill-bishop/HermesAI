use anyhow::Result;
use crate::services::node_client::NodeClient;
use tracing::info;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct SessionManager {
    pub nodes: Arc<RwLock<HashMap<String, String>>>,
    pub client: NodeClient,
}

impl SessionManager {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("token-will".into(), "http://pty-will:8080".into());
        map.insert("token-grace".into(), "http://pty-grace:8080".into());
        Self { nodes: Arc::new(RwLock::new(map)), client: NodeClient::new() }
    }

    async fn resolve(&self, token: &str) -> Result<String> {
        let map = self.nodes.read().await;
        map.get(token).cloned().ok_or_else(|| anyhow::anyhow!("invalid token"))
    }

    pub async fn write_and_read(&self, token: &str, cmd: String) -> Result<Vec<String>> {
        let url = self.resolve(token).await?;
        info!("Executing '{}' on {}", cmd, url);
        self.client.post_cmd(&url, &cmd).await
    }

    pub async fn read_latest(&self, token: &str) -> Result<Vec<String>> {
        let url = self.resolve(token).await?;
        info!("Reading stream from {}", url);
        self.client.read_stream(&url).await
    }
}
