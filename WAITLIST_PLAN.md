# 🧩 DropCode.org – Waitlist Feature Plan

**Goal:** Make the waitlist fully functional so users can sign up through the frontend, get validated and stored in the database, and optionally trigger notifications.

---

## ⚙️ 1. Backend Completion (Flask)
**Status:** ✅ Mostly complete
**Goal:** Harden, polish, and prepare for production.

### Tasks:
- [ ] Add CORS configuration for `/api/waitlist` (allow frontend origin).
- [ ] Validate email format server-side.
- [ ] Add optional `source` param (to track marketing campaigns).
- [ ] Add logging for each new signup.
- [ ] (Optional) Send confirmation email or Slack webhook on new entries.

**Files:**
- `apps/execution-sandbox/sandbox_server/routes/waitlist.py`
- `apps/execution-sandbox/sandbox_server/__init__.py`
- `apps/execution-sandbox/sandbox_server/config.py`

---

## 🖥️ 2. Frontend Integration (Angular 20)
**Goal:** Create a clean “Join Waitlist” experience in `dropcode-client`.

### Structure:
```
apps/dropcode-client/src/app/waitlist/
├── waitlist.component.html
├── waitlist.component.ts
├── waitlist.component.scss
└── waitlist.service.ts
```

### Tasks:
- [ ] Create `WaitlistService` with method `joinWaitlist(email: string)` → `POST /api/waitlist`.
- [ ] Create `WaitlistComponent` with form (email field + submit button).
- [ ] Display success/error messages.
- [ ] Add route `/waitlist` → `WaitlistComponent`.
- [ ] Add CTA in navbar or hero section linking to `/waitlist`.
- [ ] Handle duplicate email gracefully (`409` response).

---

## 🌐 3. Deployment & Integration
### Backend:
- [ ] Add `/api/waitlist` to Nginx reverse proxy.
- [ ] Expose Flask’s `/api/waitlist` publicly (HTTPS).
- [ ] Run `flask db upgrade` to ensure table is created.

### Frontend:
- [ ] Update `environment.prod.ts` to point to live backend.
- [ ] Deploy Angular build to Nginx static dir.

---

## 📬 4. Optional Enhancements
| Feature | Description | Priority |
|----------|--------------|-----------|
| Email confirmation | Send welcome email via SendGrid | Medium |
| Slack webhook | Notify internal team on new signup | Medium |
| Source tracking | Add hidden field for `utm_source` or referrer | High |
| Admin dashboard | List all waitlist users (`/admin/waitlist`) | Low |
| Pagination/API limit | Limit API results when fetching waitlist entries | Low |

---

## ✅ 5. Acceptance Criteria
- [ ] User can visit `/waitlist` and submit a valid email.
- [ ] Duplicate email → “You’re already on the list!” message.
- [ ] Successful email → “Welcome to the waitlist!” message.
- [ ] Entry appears in PostgreSQL table `waitlist`.
- [ ] Works in dev & production.