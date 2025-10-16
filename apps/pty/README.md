# isolated-exec (patched MVP)
Updated to compile cleanly with nix 0.29 and axum 0.7.

- fixed nix `dup2` and fd issues
- switched to `axum::serve`
- all async RwLock guards now awaited
- PTY I/O uses `try_io` for read/write
