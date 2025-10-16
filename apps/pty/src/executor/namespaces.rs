pub fn setup_if_enabled(enable: bool) -> anyhow::Result<()> {
    if enable {
        // In real setup: unshare, mount, drop caps, etc.
        // Left minimal to ensure portability in dev environments.
        tracing::warn!(target: "ns", "namespaces requested, but MVP leaves no-op to avoid privilege issues");
    }
    Ok(())
}
