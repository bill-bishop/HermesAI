use anyhow::Result;
use reqwest::Client;
use tracing::debug;

#[derive(Clone)]
pub struct NodeClient {
    pub http: Client,
}

impl NodeClient {
    pub fn new() -> Self {
        Self { http: Client::new() }
    }

    pub async fn post_cmd(&self, url: &str, cmd: &str) -> Result<Vec<String>> {
        debug!("POST to PTY node {}", url);
        let resp = self.http
            .post(format!("{}/exec", url))
            .json(&serde_json::json!({ "cmd": ["/bin/bash", "-lc", cmd] }))
            .send()
            .await?;

        let text = resp.text().await?;
        Ok(vec![text])
    }

    pub async fn read_stream(&self, url: &str) -> Result<Vec<String>> {
        debug!("GET from PTY node {}", url);
        let resp = self.http.get(format!("{}/stream", url)).send().await?;
        Ok(vec![resp.text().await?])
    }
}
