# DropCode Monorepo

DropCode is a **Dropbox-style app for developers** that provides seamless file sync, GitHub integration, and GPT-powered coding assistance. It is designed as a monorepo for easier development and collaboration.

---

## Features
- **File Sync (Dropbox-style)**: Real-time syncing of local workspaces with conflict resolution.
- **GitHub Integration**: Auto-committing, PR workflows, and repo mirroring.
- **GPT Hooks**: Connect workspaces with custom GPT models for refactoring, testing, and documentation.
- **Collaboration**: Shared workspaces, inline reviews, and team coding support.

---

## Monorepo Structure
```
dropcode-monorepo/
  apps/
    client/        # Local desktop/CLI client
    server/        # Central sync + API server
  packages/
    sync-engine/   # File sync logic (watchers, delta transfer)
    gpt-hooks/     # GPT integration SDK and proxy tools
    github-integration/ # GitHub workflow automation
  docs/
    PLAN.md        # Detailed architecture and planning document
  README.md        # You are here
```

---

## Roadmap
1. Implement basic **Sync Engine** for local file watching.
2. Build **Server API** for handling sync and metadata.
3. Connect **Client App** to the server with secure auth.
4. Add **GitHub integration** (auto-commit + PR support).
5. Integrate **GPT hooks** for AI-driven development tasks.
6. Expand into **collaboration features** (shared workspaces, reviews).

---

## Status
🚧 Early planning phase — scaffolding complete, implementation not started.