
use super::Cache;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct MemoryCache { inner: Arc<DashMap<String, u64>> }

impl MemoryCache {
    pub fn new() -> Self { Self { inner: Arc::new(DashMap::new()) } }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get_last_seq(&self, ws_key: &str) -> Option<u64> {
        self.inner.get(ws_key).map(|v| *v)
    }
    async fn set_last_seq(&self, ws_key: &str, seq: u64) -> anyhow::Result<()> {
        self.inner.insert(ws_key.to_string(), seq);
        Ok(())
    }
}
