use anyhow::{Result, Context};
use futures_util::StreamExt;
use regex::Regex;
use reqwest::{Client, Response};
use std::sync::Arc;
use serde::Deserialize;
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

    /// Live tail: fetch last ~50 lines from the PTY node for the last job.
    pub async fn get_terminal_tail_live(&self, node_url: &str, token: &str) -> Result<String> {
        use serde::Deserialize;
        use tokio::time::{timeout, Duration};
        use futures_util::StreamExt;

        #[derive(Deserialize)]
        struct Frame { t: Option<String>, d: Option<String> }
        #[derive(Deserialize)]
        struct Status { state: String, exit_code: Option<i32>, seq_latest: u64 }

        // ---- resolve job ----
        let last_job_id = {
            let cache = self.cache.read().await;
            match cache.get(token).and_then(|s| s.last_job_id.clone()) {
                Some(j) => j,
                None => return Ok("(no active session)".into()),
            }
        };

        // ---- query latest status ----
        let status_url = format!("{}/status/{}", node_url.trim_end_matches('/'), last_job_id);
        let status: Status = self.http.get(&status_url).send().await?
            .error_for_status()?
            .json().await?;

        // ---- open stream near the end ----
        let from = status.seq_latest.saturating_sub(800);
        let stream_url = format!("{}/stream/{}?from={}", node_url.trim_end_matches('/'), last_job_id, from);
        let resp = self.http.get(&stream_url).send().await?;
        let mut stream = resp.bytes_stream();

        // ---- collect up to 10s or until exit ----
        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();
        let mut exit_str = String::from("still running...");

        let _ = timeout(Duration::from_secs(10), async {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        for line in String::from_utf8_lossy(&bytes).lines() {
                            if line.trim().is_empty() { continue; }

                            if let Ok(frame) = serde_json::from_str::<Frame>(line) {
                                match frame.t.as_deref() {
                                    Some("stdout") => if let Some(d) = frame.d { stdout_buf.push_str(&d); },
                                    Some("stderr") => if let Some(d) = frame.d { stderr_buf.push_str(&d); },
                                    Some("event") => if let Some(d) = frame.d {
                                        if d.starts_with("exit:") {
                                            exit_str = d.trim_start_matches("exit:").to_string();
                                            let mut cache = self.cache.write().await;
                                            if let Some(s) = cache.get_mut(token) { s.running = false; }
                                            return; // exit early
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("stream read error: {}", e);
                        break;
                    }
                }
            }
            // timeout hit → mark not running
            let mut cache = self.cache.write().await;
            if let Some(s) = cache.get_mut(token) { s.running = false; }
        }).await;

        // ---- clean + tail ----
        let ansi = Regex::new(r"\x1B\[[0-9;]*[A-Za-z]").unwrap();
        let cleaned = ansi.replace_all(&format!("{}{}", stdout_buf, stderr_buf), "").to_string();
        let lines: Vec<&str> = cleaned.lines().collect();
        let tail = lines[lines.len().saturating_sub(50)..].join("\n");

        let mut out = tail;
        if exit_str == "still running..." {
            out.push_str("\n(... process still running ...)\n");
            let mut cache = self.cache.write().await;
            if let Some(s) = cache.get_mut(token) { s.running = true; }
        } else {
            out.push_str(&format!("\n(Exit code: {})\n", exit_str.trim()));
        }

        Ok(out)
    }


    
    pub async fn get_file(&self, node_url: &str, path: &str) -> Result<String> {
        let url = format!("{}/sandbox/{}", node_url.trim_end_matches('/'), path);
        let resp = self.http.get(&url).send().await?;
        if resp.status().is_success() {
            Ok(resp.text().await?)
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("GET {url} failed: {} {}", status, text);
        }
    }

    pub async fn write_file(&self, node_url: &str, path: &str, content: &str) -> Result<String> {
        let url = format!("{}/sandbox/{}", node_url.trim_end_matches('/'), path);
        let body = serde_json::json!({ "content": content });
        let resp = self.http.post(&url).json(&body).send().await?;
        if resp.status().is_success() {
            Ok(resp.text().await?)
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("POST {url} failed: {} {}", status, text);
        }
    }
}
// UPDATE
