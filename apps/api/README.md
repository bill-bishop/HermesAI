
# Agent API (control plane)

Endpoints:
- POST /terminal { cmd }
- GET /terminal
- POST /terminal/signal { signal }

Auth: Bearer <agent_api_token> → resolved to (user, workspace, node_url) by a hardcoded map in `state.rs` for MVP.
