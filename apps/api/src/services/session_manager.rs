use std::collections::HashMap;
use std::fs;
use anyhow::Result;
use tracing::info;

use crate::services::node_client::NodeClient;

// UPDATE
#[derive(Clone)]
pub struct SessionManager {
    pub client: NodeClient,
    pub token_map: std::collections::HashMap<String, String>, // token -> node_url
}

impl SessionManager {
    pub fn new() -> Self {
        // Mounted volume
        let path = "/4d24721f7fe5084b6f4e0c93eca86954/active.json";

        let token_map: HashMap<String, String> = fs::read_to_string(path)
            .ok()
            .and_then(|txt| serde_json::from_str(&txt).ok())
            .unwrap_or_else(|| {
                eprintln!("⚠️  Warning: could not read {}", path);
                let m = HashMap::new();
                m
            });

        let client = NodeClient::new();
        Self { client, token_map }
    }

    pub fn resolve_node(&self, token: &str) -> Option<String> {
        self.token_map.get(token).cloned()
    }

    pub async fn execute(&self, token: &str, cmd: &str) -> Result<String> {
        if let Some(node) = self.resolve_node(token) {
            info!("Executing '{}' on {}", cmd, node);
            let output = self.client.post_exec(&node, token, cmd).await?;
            info!("Created job {}", output);
            Ok(output)
        } else {
            anyhow::bail!("unknown token {token}");
        }
    }

    pub async fn get_terminal_tail(&self, token: &str) -> Result<String> {
        self.client.get_terminal_tail(token).await
    }
}
// UPDATE
