# isolated-exec (Namespaces + PTY MVP)

A minimal, working server that:
- runs non-interactive commands and interactive shells,
- streams output as NDJSON with resume by sequence,
- supports PTY sessions (write/resize/close),
- stubs namespace setup (can be enabled later).

> NOTE: Linux namespaces often require privileges. This MVP leaves namespaces **disabled by default** via config to compile/run everywhere. Flip `namespaces.enable=true` once host allows it.

## Endpoints
- `POST /exec`
- `GET /stream/:job_id?from=SEQ`
- `GET /status/:job_id`
- `POST /cancel/:job_id`
- `POST /sessions`
- `GET /sessions/:id/stream?from=SEQ`
- `POST /sessions/:id/write`
- `POST /sessions/:id/resize`
- `POST /sessions/:id/close`
