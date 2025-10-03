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

## Commit Process (Important for Agents)
When making changes, agents must commit in **two steps** so the CI daemon can detect, push, and redeploy:
1. Commit changes inside **`/sandbox/apps/execution-sandbox`** (the submodule).
2. Commit the updated **submodule pointer** inside **`/sandbox`** (the monorepo root).

This ensures the CI daemon sees both repos updated, pushes them upstream, and triggers a redeploy of the application.

## Next Steps
- **Grace’s Tasks (Product/UX):**
  - Draft new homepage copy.
  - **[CURRENT TASK] Create homepage wireframe.**
  - Draft new feature page copy.
  - Create feature page wireframe.
  - Draft new pricing page copy.
  - Create pricing page wireframe.
  - Investigate Socket.IO disconnect / cannot send message errors (frontend error handling & UX impact).

- **Bill’s Tasks (Architecture/Backend/Engineering):**
  - Flesh out Home/Features/Pricing components with real app content.
  - Add tests for `HomeComponent`, `FeaturesComponent`, and `PricingComponent`.
  - Complete in-house auth (local user login/register backend logic).
  - Extend ThirdPartyAuthComponent with Google & Facebook providers.
  - Add refresh token support (currently only access tokens).
  - Harden Nginx config (CSP headers, rate limiting, etc.).
  - CI/CD pipeline for building Angular + deploying Flask/Nginx container.
  - Build **User** and **Workspace** tables in the backend DB.
  - Add Kubernetes config + healthcheck endpoints for container orchestration and monitoring.
  - **[CURRENT TASK] Integrate user authentication with workspace selection:**
    - Add `workspace_id` to JWT tokens.
    - Ensure `/auth/me` includes the selected workspace.
    - Update **Nginx ↔ Flask handoff** so that Nginx routes requests to a container URL based on `workspace_id`.
    - Implement deterministic container routing:
      - Currently all users share one global sandbox (`http://sandbox:8080`).
      - Spin up **one container per test user** with a deterministic internal URL.
      - Update Nginx upstreams or reverse proxy logic to map `workspace_id → container URL`.
  - **Add Git Hook for Server Reload:**
    - Implement a post-receive (or CI/CD hook) that automatically reloads the Flask + Nginx server when new commits are pushed.
    - Ensure safe zero-downtime reload (e.g., `nginx -s reload` and Flask/Gunicorn reload).
    - Security: whitelist repo origin, ensure only trusted pushes trigger reload.

---

## MVP Spec: User Authentication + Workspace Integration

For the MVP, container lifecycle will be **manual** (admin spins up per-user containers). Our goal is to wire up the backend + Nginx so that each authenticated user is bound to their own container.

### 1. Database Changes
- Extend **User** table:
  - `id`: int, primary key.
  - `email`, `password_hash`, etc.
- New **Workspace** table:
  - `id`: int, primary key.
  - `user_id`: FK → User.id.
  - `container_url`: string (e.g., `http://sandbox-user1:8080`).
  - `created_at`, `updated_at`.

### 2. Authentication Flow Changes
- On login/register:
  - Fetch or create a `Workspace` for the user.
  - Include `workspace_id` in the JWT payload.
  - Issue cookie with both `sub` (user_id) and `workspace_id`.
- `/auth/me`:
  - Return `{ user, workspace }` where workspace includes container_url.

### 3. Backend Changes (Flask)
- Add `Workspace` model in `models.py`.
- Modify `auth.py` and registration flow to auto-create a workspace row.
- Update JWT generation:
  ```python
  additional_claims = { "workspace_id": workspace.id }
  access_token = create_access_token(identity=user.id, additional_claims=additional_claims)
  ```
- Update `/auth/me` to return:
  ```json
  {
    "id": 123,
    "email": "user@example.com",
    "workspace": {
      "id": 456,
      "container_url": "http://sandbox-user1:8080"
    }
  }
  ```

### 4. Nginx Changes
- Define upstream blocks for each manually spun-up container:
  ```nginx
  upstream workspace_456 {
    server sandbox-user1:8080;
  }
  ```
- Use a **Lua script** or **nginx map** to select upstream based on JWT `workspace_id` claim:
  - MVP: Hardcode mapping in nginx.conf.
  - Later: dynamic lookup (e.g., Redis or shared dict).
- Proxy requests to the correct container:
  ```nginx
  location /api/execute/ {
    proxy_pass http://workspace_456;
  }
  ```

### 5. Frontend Changes (Angular)
- Update `AuthService`:
  - Store both `user` and `workspace` in `currentUser$`.
- Expose `workspace.container_url` so the UI can route API calls accordingly.
- Ensure guards wait for workspace info before loading Home/Features/Pricing.

### 6. Manual Container Management (for MVP)
- Admin spins up containers manually, named predictably (e.g., `sandbox-user1`).
- Insert `container_url` into DB for each workspace.
- Nginx config updated manually with corresponding upstream.

### 7. Future Extensions (post-MVP)
- Automate container lifecycle (spin up/down on demand).
- Dynamic Nginx upstreams via Lua + Redis.
- Support multiple workspaces per user.

---
This PLAN.md serves as context for future dev cycles and as a quick bootstrapping guide for new engineers or AI copilots.