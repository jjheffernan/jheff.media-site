# Release readiness rubric

Structured assessment for jheff.media-site. Re-score after major milestones.

**Last assessed:** 2026-10-29

## Scoring scale (1–5)

| Score | Label | Meaning |
|-------|-------|---------|
| 1 | Not ready | Blocks any real use; missing fundamentals or known critical flaws |
| 2 | Early | Dev/demo only; major gaps in this category |
| 3 | Usable | Core flows work with documented caveats; acceptable for personal/staging |
| 4 | Production-capable | Meets expectations for a small production deploy with minor follow-ups |
| 5 | Mature | Hardened, tested, and operable without heroic effort |

Percent equivalent: `(score / 5) × 100%` (e.g. 3 = 60%).

---

## Current scores

| Category | Score | % | Summary |
|----------|-------|---|---------|
| **Security** | 3 | 60% | Solid JWT+bcrypt+session auth, in-memory auth and contact limits, and login-time 2FA. CSP is still missing |
| **UX** | 3 | 60% | Feed, print select, and owned dates work. Galleries still lack edit/delete, and checkout does not charge a card |
| **Tests** | 2 | 40% | Backend unit tests and both suites in CI. No HTTP tests against Mongo and no real browser test |
| **12 Factor** | 4 | 80% | Docker + env config; runtime JWT secret; pinned Rust images; `jheffmedia-*` docker tags |
| **Performance** | 3 | 60% | Actix + async Mongo appropriate; WASM payload, no caching, thumbnail proxy untested under load |

**Overall weighted average:** 3.0 / 5 (60%) — **acceptable for personal/staging deploy; not client-production ready without prioritized hardening and feature completion.**

---

## Security (3 / 5)

### Criteria

| Criterion | Weight | Status | Notes |
|-----------|--------|--------|-------|
| Authentication | High | ✅ | JWT (HS256) + server-side `loginSession` in MongoDB |
| Password storage | High | ✅ | bcrypt; min 8 chars |
| Authorization | High | ⚠️ | Admin routes enforce role; profile not gated; admin panel is hidden unless `role == admin` |
| Transport / edge | High | ⚠️ | HTTPS/HSTS documented for reverse proxy, not enforced in app |
| Headers | Medium | ✅ | `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` in `main.rs` |
| CORS | Medium | ✅ | Explicit origins via `YEW_FULLSTACK_CORS_ORIGINS` (not wildcard) |
| 2FA | Medium | ✅ | Enroll/confirm/disable API; login requires a valid code before creating a session |
| Abuse resistance | Medium | ⚠️ | In-memory limits: 8 auth attempts and 5 contact posts per IP per 10 minutes. Resets on process restart |
| CSP / XSS | Medium | ❌ | JWT in session storage; no Content-Security-Policy |
| Secrets management | High | ✅ | Runtime JWT via `YEW_FULLSTACK_JWT_SECRET_FILE` or hex env; dev `secret.key` fallback |

### Evidence

- `backend/src/utils/auth.rs`, `backend/src/models/user_token.rs`
- `docs/authentication.md` hardening backlog
- `backend/src/main.rs` middleware

### To reach 4

- [x] Runtime JWT secret (env/volume mount, not compile-time embed)
- [x] Rate limiting on auth + contact endpoints
- [x] Enforce TOTP at login when `totp_enabled`
- [ ] CSP tuned for WASM + embeds
- [x] Hide admin UI unless `role == admin`

### To reach 5

- [ ] Security audit / dependency scanning in CI
- [ ] Email change mailer wired before production enable
- [ ] Penetration-style auth tests

---

## UX (3 / 5)

### Criteria

| Criterion | Weight | Status | Notes |
|-----------|--------|--------|-------|
| Navigation / routing | High | ✅ | 15+ Yew routes (`frontend/src/routes/mod.rs`) |
| Visual design | Medium | ✅ | Tailwind v4, shared UI components |
| Homepage / feed | High | ✅ | `MediaFeedScroll` + `/api/media/feed` |
| Auth UX | High | ✅ | Login, signup, logout, account settings |
| Shoots / galleries | High | ⚠️ | Admin image upload and detail pages; still no edit/delete or batch upload |
| Client delivery | High | ⚠️ | Detail pages can select stills; checkout stores one size and an unpaid total. No card charge |
| Social integrations | Medium | ⚠️ | Empty Instagram config returns 204 and the embed widget is gone. Live posts still need keys or static JSON |
| Contact / booking | Medium | ⚠️ | Contact emails when SMTP is set; schedule lists future Mongo dates and can still embed a calendar |
| Accessibility / mobile | Low | ⚠️ | Not formally assessed |
| Error / loading states | Medium | ⚠️ | Spinners exist; inconsistent across pages |

### Evidence

- `frontend/src/components/feed/media_feed_scroll.rs`
- `docs/TODO.md` site functionality section
- `frontend/src/routes/account.rs` (2FA UI)

### To reach 4

- [x] Real shoot/gallery media upload and detail pages with assets
- [ ] Immich or static feed configured for production homepage
- [x] Contact notification (email or webhook)
- [x] Schedule page lists owned dates; external calendar embed stays optional

### To reach 5

- [ ] Print checkout end-to-end (size and unpaid total exist; card payment does not)
- [x] Search/filter unified feed
- [ ] Polish 2FA enrollment UX (QR display)

---

## Tests (2 / 5)

### Criteria

| Criterion | Weight | Status | Notes |
|-----------|--------|--------|-------|
| Unit tests | High | ⚠️ | Backend suite covers contact, auth, print prices, and schedule filtering. No HTTP tests against Mongo |
| CI | High | ✅ | Trunk build, backend build, both test suites in `.github/workflows/rust.yml` |
| Integration tests | High | ❌ | No HTTP handler tests against test MongoDB |
| E2E / browser | Medium | ❌ | `frontend/tests/web.rs` is placeholder (`1+1==2`) |
| Lint / static analysis | Medium | ⚠️ | 12 clippy warnings (TODO); no clippy in CI |
| Regression safety | Medium | ❌ | No API contract tests for 20+ endpoints |

### Evidence

```bash
cargo test -p jheffmedia-site-backend   # binary crate; last green run was 21 passed
cargo test -p jheffmedia-site-frontend  # 3 passed (+ 0 browser tests)
```

### To reach 3

- [x] Add frontend tests to CI (host `cargo test` in `.github/workflows/rust.yml`)
- [ ] Fix clippy warnings; add `cargo clippy` to CI
- [ ] One smoke test per critical API (`login`, `media/feed`) — `scripts/smoke.sh` covers homepage + login reachability

### To reach 4

- [ ] Integration tests with `testcontainers` or embedded Mongo
- [ ] Replace `web.rs` with at least one route render or API contract test

### To reach 5

- [ ] E2E smoke (headless) for login + homepage feed
- [ ] Coverage threshold or critical-path checklist in CI

---

## 12 Factor (4 / 5)

| Factor | Status | Notes |
|--------|--------|-------|
| I Codebase | ✅ | Single repo, Cargo workspace |
| II Dependencies | ⚠️ | `package-lock.json` is committed. `Cargo.lock` is gitignored |
| III Config | ✅ | `YEW_FULLSTACK_*` env vars; runtime JWT secret |
| IV Backing services | ✅ | MongoDB as attachable service in Compose |
| V Build/release | ✅ | Multi-stage `Dockerfile`, `scripts/prod.sh` |
| VI Processes | ✅ | Stateless Actix workers |
| VII Port binding | ✅ | `YEW_FULLSTACK_HOST` / `PORT` |
| VIII Concurrency | ✅ | Actix async model |
| IX Disposability | ⚠️ | Not explicitly documented |
| X Dev/prod parity | ✅ | `scripts/dev.sh`, install targets |
| XI Logs | ✅ | `env_logger` / `RUST_LOG` |
| XII Admin processes | ✅ | Admin API, install scripts, one-off migrations manual |

### Gaps

- Legacy `YEW_FULLSTACK_*` env prefix (docker tags renamed to `jheffmedia-*`)
- Health check endpoint + Compose `healthcheck` not yet added

### To reach 4

- [x] Pin all Docker Rust images (`Dockerfile` already uses `1.85-bookworm`; align dev `backend/Dockerfile`)
- [x] Runtime JWT secret via env or mounted file
- [x] Rename docker project tags to `jheffmedia-*`

### To reach 5

- [ ] Health check endpoint + Compose `healthcheck`
- [ ] Documented graceful shutdown and backup/restore for MongoDB

---

## Performance (3 / 5)

### Criteria

| Criterion | Weight | Status | Notes |
|-----------|--------|--------|-------|
| Server framework | High | ✅ | Actix 4 — top-tier throughput in Rust ecosystem |
| Database access | High | ✅ | MongoDB 3.x async throughout |
| Frontend delivery | Medium | ⚠️ | WASM bundle size; no code-splitting documented |
| Caching | Medium | ❌ | No CDN/cache headers strategy for static or thumbnails |
| External APIs | Medium | ⚠️ | Immich/IG/YT proxy; load not tested |
| Asset pipeline | Medium | ✅ | Thumbnails proxied server-side (keys not exposed) |

### Stack comparison note

Per [rust-web-framework-comparison](https://github.com/flosse/rust-web-framework-comparison), Actix and Yew remain active, stable-Rust choices. Alternatives (Axum, Leptos, Dioxus) may offer ecosystem or SSR benefits but do not inherently outperform this stack for this workload without a full rewrite.

### To reach 4

- [ ] Cache-Control for static assets and thumbnail proxy
- [ ] Verify Immich thumbnail proxy under expected traffic
- [ ] `trunk build --release` size budget documented

### To reach 5

- [ ] Load test baseline for homepage feed + auth
- [ ] Optional Redis/cache for hot feed paths

---

## How to use this rubric

1. Score each category after a release milestone or quarterly review.
2. Record evidence links (commits, test output, deploy logs).
3. Track "To reach N" checklists in `docs/TODO.md` or link issues.
4. **Release gate suggestion:** average ≥ 3.5 with no category below 3 for public production; average ≥ 3 acceptable for personal/staging deploy.

## Related docs

- [TODO.md](./TODO.md) — actionable backlog
- [design.md](./design.md) — page behavior and open questions
- [architecture.md](./architecture.md) — system layout
- [deployment-proxmox.md](./deployment-proxmox.md) — production deploy
- [authentication.md](./authentication.md) — auth security notes
