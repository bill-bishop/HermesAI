
use async_trait::async_trait;

#[async_trait]
pub trait Cache {
    async fn get_last_seq(&self, ws_key: &str) -> Option<u64>;
    async fn set_last_seq(&self, ws_key: &str, seq: u64) -> anyhow::Result<()>;
}

pub mod memory;
pub use memory::MemoryCache;
