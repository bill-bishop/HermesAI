
use anyhow::Result;
use crate::{middleware::auth::AgentContext, models::{terminal::*, node::NodeSessionResponse}};
use tracing::{info};

#[derive(Clone)]
pub struct SessionRef { pub id: String, pub created: bool, pub ws_key: String }

impl super::super::state::SessionManager {
    pub async fn ensure_session(&self, agent: &AgentContext) -> Result<SessionRef> {
        let key = self.ws_key(&agent.user_id, &agent.workspace_id);
        if let Some(existing) = self.sessions.get(&key) {
            return Ok(SessionRef { id: existing.clone(), created: false, ws_key: key });
        }
        // create
        let NodeSessionResponse { session_id, .. } = self.node.create_session(&agent.node_url, self.cfg.default_cols, self.cfg.default_rows).await?;
        self.sessions.insert(key.clone(), session_id.clone());
        info!(session_id, "created session");
        Ok(SessionRef { id: session_id, created: true, ws_key: key })
    }

    pub async fn write_then_read(&self, agent: &AgentContext, cmd: String, wait_ms: u64) -> Result<PostTerminalResponse> {
        let sess = self.ensure_session(agent).await?;
        self.node.write(&agent.node_url, &sess.id, &(cmd + "\r")).await?;
        let from = self.cache.get_last_seq(&sess.ws_key).await.unwrap_or(0);
        let frames = self.node.long_poll_stream(&agent.node_url, &sess.id, from, wait_ms).await?;
        let next = frames.last().map(|f| f.seq).unwrap_or(from);
        if !frames.is_empty() { self.cache.set_last_seq(&sess.ws_key, next).await?; }
        Ok(PostTerminalResponse { created: sess.created, running: true, frames, next_from: next, advice: Some("call GET /terminal for additional output".into()) })
    }

    pub async fn read_new_or_tail(&self, agent: &AgentContext, wait_ms: u64, tail_n: usize) -> Result<GetTerminalResponse> {
        let sess = self.ensure_session(agent).await?;
        let from = self.cache.get_last_seq(&sess.ws_key).await.unwrap_or(0);
        let frames = self.node.long_poll_stream(&agent.node_url, &sess.id, from, wait_ms).await?;
        if !frames.is_empty() {
            let next = frames.last().unwrap().seq;
            self.cache.set_last_seq(&sess.ws_key, next).await?;
            return Ok(GetTerminalResponse { running: true, frames, tail: None, message: None, next_from: next });
        }
        // No new frames — synthesize a tail from our last seen position
        let start = from.saturating_sub(tail_n as u64);
        // We don't have a dedicated tail endpoint; just ask from=start with small budget
        let tail_frames = if start < from {
            self.node.long_poll_stream(&agent.node_url, &sess.id, start, 200).await.unwrap_or_default()
        } else { vec![] };
        Ok(GetTerminalResponse {
            running: true, frames: vec![],
            tail: Some(tail_frames.into_iter().filter(|f| f.seq <= from).collect()),
            message: Some("--- NO NEW TERMINAL OUTPUT. PREVIOUS OUTPUT BELOW ---".into()),
            next_from: from
        })
    }

    pub async fn signal(&self, agent: &AgentContext, sig: String) -> Result<()> {
        let sess = self.ensure_session(agent).await?;
        self.node.signal(&agent.node_url, &sess.id, sig).await
    }
}
