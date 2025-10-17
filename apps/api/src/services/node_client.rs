
use anyhow::Result;
use reqwest::{Client, StatusCode};
use futures_util::StreamExt;
use crate::{models::{node::*, common::StreamFrame}, util::Deadline};
use tracing::{warn};

#[derive(Clone)]
pub struct NodeClient {
    http: Client,
    node_secret: String,
}

impl NodeClient {
    pub fn new(node_secret: String) -> Self {
        let http = Client::builder().build().unwrap();
        Self { http, node_secret }
    }

    fn auth_hdr(&self) -> (&'static str, String) {
        ("x-node-secret", self.node_secret.clone())
    }

    pub async fn create_session(&self, base: &str, cols: u16, rows: u16) -> Result<NodeSessionResponse> {
        let url = format!("{}/sessions", base.trim_end_matches('/'));
        let req = NodeSessionRequest{ mode:"interactive".into(), profile:"bash".into(), cols, rows };
        let resp = self.http.post(url).header(self.auth_hdr().0, self.auth_hdr().1.clone()).json(&req).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("node create_session failed: {} {}", status, text);
        }
        Ok(resp.json().await?)
    }

    pub async fn write(&self, base: &str, session_id: &str, data: &str) -> Result<()> {
        let url = format!("{}/sessions/{}/write", base.trim_end_matches('/'), session_id);
        let req = NodeWriteRequest{ data: data.to_string() };
        let resp = self.http.post(url)
            .header(self.auth_hdr().0, self.auth_hdr().1.clone())
            .json(&req)
            .send()
            .await?;

        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("node write failed: {} {}", status, text);
        }

        Ok(())
    }

    /// Long-poll NDJSON stream from `from` for up to `wait_ms`.
    pub async fn long_poll_stream(&self, base: &str, session_id: &str, from: u64, wait_ms: u64) -> Result<Vec<StreamFrame>> {
        let url = format!("{}/sessions/{}/stream?from={}", base.trim_end_matches('/'), session_id, from);
        let resp = self.http.get(url).header(self.auth_hdr().0, self.auth_hdr().1.clone()).send().await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("node stream failed: {} {}", status, text);
        }
        let mut stream = resp.bytes_stream();
        let mut buf = Vec::new();
        let mut frames = Vec::new();
        let deadline = Deadline::new(wait_ms);

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buf.extend_from_slice(&chunk);

            // split on newlines
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line = buf.drain(..=pos).collect::<Vec<u8>>();
                if let Ok(text) = std::str::from_utf8(&line) {
                    let trimmed = text.trim();
                    if trimmed.is_empty() { continue; }
                    match serde_json::from_str::<StreamFrame>(trimmed) {
                        Ok(f) => frames.push(f),
                        Err(e) => warn!("bad ndjson: {} :: {}", trimmed, e),
                    }
                }
            }

            if deadline.exceeded() { break; }
        }

        // If nothing arrived, return empty; caller decides to tail or not.
        Ok(frames)
    }

    pub async fn signal(&self, base: &str, session_id: &str, signal: String) -> Result<()> {
        // let url = format!("{}/sessions/{}/write", base.trim_end_matches('/'), session_id);
        // For MVP, route as Ctrl-C by data = \x03 if INT, else ignore.
        let data = if signal.to_uppercase() == "INT" { "\u{0003}" } else { "" }.to_string();
        if data.is_empty() { return Ok(()); }
        self.write(base, session_id, &data).await
    }
}
