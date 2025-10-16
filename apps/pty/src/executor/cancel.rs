use crate::state::JobHandle;
use parking_lot::Mutex;
use std::sync::Arc;

pub async fn cancel_job(_h: Arc<Mutex<Option<tokio::process::Child>>>) -> anyhow::Result<()> {
    // TODO: store child handle; send SIGTERM; fallback SIGKILL
    Ok(())
}
