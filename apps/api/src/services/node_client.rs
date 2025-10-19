use anyhow::{Result, Context};
use futures_util::StreamExt;
use regex::Regex;
use reqwest::{Client, Response};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use crate::models::terminal::ExecResponse;

// UPDATE
#[derive(Clone)]
pub struct NodeClient {
    pub http: Client,
    pub ansi_re: Regex,
    pub cache: Arc<RwLock<std::collections::HashMap<String, NodeState>>>,
}

#[derive(Clone, Default)]
pub struct NodeState {
    pub backlog: String,
    pub last_job_id: Option<String>,
    pub running: bool,
}

impl NodeClient {
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                // .timeout(None)
                .build()
                .expect("reqwest client build failed"),
            ansi_re: Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap(),
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Send command to PTY
    pub async fn post_exec(&self, node_url: &str, token: &str, cmd: &str) -> Result<String> {
        let url = format!("{}/exec", node_url.trim_end_matches('/'));
        debug!("POST to PTY node {}", url);

        let body = serde_json::json!({ "cmd": [cmd] });
        let resp = self.http.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("PTY exec failed: {} {}", status, text);
        }

        let exec_resp: ExecResponse = resp.json().await?;
        let job_id = exec_resp.job_id.clone();
        
        // mark node as active
        {
            let mut cache = self.cache.write().await;
            let state = cache.entry(token.to_string()).or_default();
            state.running = true;
            state.last_job_id = Some(job_id.clone());
        }

        // start background reader
        let client = self.clone();
        let token = token.to_string();
        let node_url = node_url.to_string();
        let retid = job_id.clone();
        tokio::spawn(async move {
            if let Err(e) = client.stream_plain_background(&node_url, &token, &job_id).await {
                info!("background stream failed: {e}");
            }
        });

        Ok(retid)
    }

    async fn stream_plain_background(&self, node_url: &str, token: &str, job_id: &str) -> Result<()> {
        let url = format!("{}/stream/{}?from=0", node_url.trim_end_matches('/'), job_id);
        info!("stream_plain_background from {}", url);

        let resp = self.http.get(&url).send().await?;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let data = String::from_utf8_lossy(&bytes);
                    let cleaned = self.ansi_re.replace_all(&data, "").to_string();
                    let mut cache = self.cache.write().await;
                    let entry = cache.entry(token.to_string()).or_default();
                    entry.backlog.push_str(&cleaned);
                    if entry.backlog.len() > 4096 {
                        let start = entry.backlog.len() - 4096;
                        entry.backlog = entry.backlog[start..].to_string();
                    }
                }
                Err(e) => {
                    info!("stream read error: {}", e);
                    break;
                }
            }
        }

        // mark session inactive
        let mut cache = self.cache.write().await;
        if let Some(entry) = cache.get_mut(token) {
            entry.running = false;
        }

        Ok(())
    }

    /// Tail of backlog (for /terminal)
    pub async fn get_terminal_tail(&self, token: &str) -> Result<String> {
        let cache = self.cache.read().await;
        if let Some(state) = cache.get(token) {
            let mut lines: Vec<&str> = state.backlog.lines().rev().take(30).collect();
            lines.reverse();
            let mut tail = lines.join("\n");
            if state.running {
                tail.push_str("\n(... process still running ...)\n");
            }
            Ok(tail)
        } else {
            Ok("(no active session)".into())
        }
    }
}
// UPDATE
