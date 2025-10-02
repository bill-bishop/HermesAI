# Project Plan & Context

This document summarizes the architecture, quirks, and next steps for the DropCode monorepo.

## Monorepo Structure
- **apps/dropcode-client**: Angular 20 frontend (standalone components, Bootstrap theme).
- **apps/execution-sandbox**: Python Flask backend + Nginx for static/UI serving and API proxy.

### Frontend (dropcode-client)
- **Angular 20** using standalone components.
- Auth flow:
  - Login / Register components (Bootstrap forms).
  - ThirdPartyAuthComponent: GitHub login (can extend to Google/Facebook).
  - AuthService: manages currentUser$ via cookie-based JWT.
  - Guards: RxJS-based, wait until auth state is resolved before redirecting.
- Routing:
  - `/login`, `/register` (public).
  - `/` → HomeComponent (requires auth).
  - `/features`, `/pricing` → placeholder pages (requires auth).
- Tests:
  - Use `ng test` (Karma + Jasmine). Jest is **not supported** in Angular 20.
  - Tests updated to match actual expectations (navbar + router-outlet).

### Backend (execution-sandbox)
- **Flask + Flask-JWT-Extended**.
- DB: SQLite (default).
- SocketIO for workspace events.
- Auth:
  - Local login/register endpoints.
  - GitHub OAuth implemented with `/auth/redirect/github` and `/auth/callback/github`.
  - Tokens issued as `auth_token` cookies (HttpOnly, Secure, SameSite=Lax).
  - `/auth/me` returns current user from cookie.
  - `/auth/logout` clears cookie.
- Nginx:
  - Serves Angular app under `/`.
  - Proxies `/api/*` → Flask.
  - Config includes `try_files $uri /index.html;` to support Angular 20 path routing.

## Angular 20 Quirks
- `jest` does not integrate cleanly. Use `ng test` instead.
- For routing, do **not** use `withHashLocation()`. Default HTML5 routing is supported, but requires nginx fallback.
- Guards must handle async properly (tri-state `undefined | null | User`).

## Adding New Backend Routes
1. Create a new file in `apps/execution-sandbox/sandbox_server/routes/`.
2. Define a Flask Blueprint, register it in `app.py`.
3. Update Nginx if you need `/api/...` paths exposed.

## Next Steps
- **Grace’s Tasks (Product/UX):**
  - Draft new homepage copy.
  - Create homepage wireframe.
  - Draft new feature page copy.
  - Create feature page wireframe.
  - Draft new pricing page copy.
  - Create pricing page wireframe.

- **Bill’s Tasks (Architecture/Backend/Engineering):**
  - Flesh out Home/Features/Pricing components with real app content.
  - Add tests for `HomeComponent`, `FeaturesComponent`, and `PricingComponent`.
  - Complete in-house auth (local user login/register backend logic).
  - Extend ThirdPartyAuthComponent with Google & Facebook providers.
  - Add refresh token support (currently only access tokens).
  - Harden Nginx config (CSP headers, rate limiting, etc.).
  - CI/CD pipeline for building Angular + deploying Flask/Nginx container.
  - Build **User** and **Workspace** tables in the backend DB.
  - Integrate user authentication with workspace selection:
    - Add `workspace_id` to JWT tokens.
    - Ensure `/auth/me` includes the selected workspace.
  - Update **Nginx ↔ Flask handoff** so that Nginx routes requests to a container URL based on `workspace_id`.
  - Implement deterministic container routing:
    - Currently all users share one global sandbox (`http://sandbox:8080`).
    - Spin up **one container per test user** with a deterministic internal URL.
    - Update Nginx upstreams or reverse proxy logic to map `workspace_id → container URL`.
  - **[CURRENT TASK] Fix ANSI formatting issues in the terminal UI.**

---
This PLAN.md serves as context for future dev cycles and as a quick bootstrapping guide for new engineers or AI copilots.