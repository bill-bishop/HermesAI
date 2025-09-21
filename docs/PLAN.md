# DropCode Monorepo Plan

## Overview
DropCode is a Dropbox-style application for developers, providing seamless file sync, GitHub integration, and GPT-powered coding assistance. It functions similarly to a local sandbox environment but adds collaboration and automation features.

---

## Architecture
- **Client App (Desktop/CLI)**
  - File system watcher for real-time sync
  - Reverse proxy for GPT integration
  - Local-first with background sync

- **DropCode Server**
  - File sync and version metadata
  - Authentication and access control
  - API for GPT hooks, GitHub integration, and user workspaces

- **Database**
  - Stores metadata (users, workspaces, versions)
  - Candidate: PostgreSQL or MongoDB

- **Object Storage**
  - Stores file diffs, snapshots, and archives
  - Candidate: S3/MinIO

---

## Core Features
- File Sync (Dropbox-style)
  - Watch local changes, debounce, sync deltas
  - Conflict resolution
- GitHub Integration
  - Auto-commit on interval or debounce
  - PR creation, mirroring workspaces
- GPT Integration
  - Local reverse proxy to custom GPT
  - Plugins for AI-driven tasks (refactors, docs, testing)
- Collaboration
  - Shared workspaces
  - Inline code reviews

---

## Security
- End-to-end encryption for sync
- OAuth for authentication (GitHub/Google)
- TLS for reverse proxy connections

---

## Frontend Options
- Electron desktop client
- CLI tool for lightweight use
- Web dashboard for management

---

## Stretch Features
- Live code sessions (VSCode Live Share-like)
- AI pair programming
- Workspace templates

---

## Monorepo Layout
```
dropcode-monorepo/
  apps/
    client/        # Electron or CLI client
    server/        # API + sync server
  packages/
    sync-engine/   # File watching + delta transfer
    gpt-hooks/     # GPT integration SDK
    github-integration/ # Auto-commit + PR tooling
  docs/
    PLAN.md        # Project planning document
```