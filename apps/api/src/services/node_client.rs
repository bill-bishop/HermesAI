use anyhow::{Result, Context};
use futures_util::StreamExt;
use regex::Regex;
use reqwest::{Client, Response};
use std::sync::Arc;
use serde_json::Value;
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
    
    /// Send command to PTY and wait up to ~10s for output
    pub async fn post_exec(&self, node_url: &str, token: &str, cmd: &str) -> Result<String> {
        use serde::Deserialize;
        use tokio::time::{timeout, Duration};
        use futures_util::StreamExt;

        #[derive(Deserialize)]
        struct Frame {
            t: Option<String>,
            d: Option<String>,
        }

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
            state.backlog.clear();                // <-- reset backlog each run
        }

        // ---- actively collect output for up to 10s ----
        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();
        let mut exit_str = String::from("still running...");

        let stream_url = format!("{}/stream/{}?from=0", node_url.trim_end_matches('/'), job_id);
        let resp = self.http.get(&stream_url).send().await?;
        let mut stream = resp.bytes_stream();

        let _ = timeout(Duration::from_secs(10), async {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        for line in String::from_utf8_lossy(&bytes).lines() {
                            if line.trim().is_empty() {
                                continue;
                            }

                            // append to cache backlog so /terminal can read it
                            {
                                let mut cache = self.cache.write().await;
                                let state = cache.entry(token.to_string()).or_default();
                                state.backlog.push_str(line);
                                state.backlog.push('\n');
                            }

                            if let Ok(frame) = serde_json::from_str::<Frame>(line) {
                                match frame.t.as_deref() {
                                    Some("stdout") => {
                                        if let Some(d) = frame.d { stdout_buf.push_str(&d); }
                                    }
                                    Some("stderr") => {
                                        if let Some(d) = frame.d { stderr_buf.push_str(&d); }
                                    }
                                    Some("event") => {
                                        if let Some(d) = frame.d {
                                            if d.starts_with("exit:") {
                                                exit_str = d.trim_start_matches("exit:").to_string();
                                                // mark stopped
                                                let mut cache = self.cache.write().await;
                                                if let Some(state) = cache.get_mut(token) {
                                                    state.running = false;
                                                }
                                                return; // end early
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                debug!("non-JSON stream fragment: {}", line);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("stream read error: {}", e);
                        break;
                    }
                }
            }

            // ensure running=false even if timeout hit
            let mut cache = self.cache.write().await;
            if let Some(state) = cache.get_mut(token) {
                state.running = false;
            }
        }).await;

        // ---- truncate if too long ----
        const LIMIT: usize = 1000;
        let truncate = |s: &mut String| {
            if s.len() > LIMIT {
                s.truncate(LIMIT);
                s.push_str("\n[...truncated]");
            }
        };
        truncate(&mut stdout_buf);
        truncate(&mut stderr_buf);

        // normalize exit
        let exit_display = if exit_str == "still running..." {
            exit_str.clone()
        } else {
            exit_str
                .trim_start_matches("Some(")
                .trim_end_matches(')')
                .trim()
                .to_string()
        };

        // ---- format summary ----
        let summary = format!(
            "STDOUT:\n{}\n\nSTDERR:\n{}\n\nEXIT CODE:\n{}",
            stdout_buf.trim_end(),
            stderr_buf.trim_end(),
            exit_display
        );

        Ok(summary)
    }


    pub async fn get_terminal_tail(&self, token: &str) -> Result<String> {
        let cache = self.cache.read().await;
        if let Some(state) = cache.get(token) {
            // Normalize backlog into clean lines
            let mut lines = Vec::new();
            let ansi = Regex::new(r"\x1B\[[0-9;]*[A-Za-z]").unwrap();

            for raw in state.backlog.lines() {
                if let Ok(v) = serde_json::from_str::<Value>(raw) {
                    if let Some(t) = v.get("t").and_then(|x| x.as_str()) {
                        if t == "stdout" || t == "stderr" {
                            if let Some(d) = v.get("d").and_then(|x| x.as_str()) {
                                let clean = ansi.replace_all(d, "").to_string();
                                lines.push(clean);
                            }
                        }
                    }
                }
            }

            // Tail last 30 lines
            let tail_n = 30;
            let len = lines.len();
            let start = len.saturating_sub(tail_n);
            let mut tail = lines[start..].join("");

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
