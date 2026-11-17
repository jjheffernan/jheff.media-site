# Release readiness rubric

Structured assessment for jheff.media-site. Re-score after major milestones.

**Last assessed:** 2026-11-15

**Beta gate:** personal/staging exit criteria live in [beta-readiness.md](./beta-readiness.md). First public tag `v0.1.0-beta.1` is a **future** step after that checklist is green — not cut from this assessment alone.

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
| **Security** | 4 | 80% | JWT+bcrypt+session, rate limits, login 2FA, admin UI gate, CSP for WASM/embeds. TLS still proxy-side; `/metrics` unauthenticated |
| **UX** | 3 | 60% | Feed, print select, schedule CRUD, and gallery/shoot admin maintenance work. Checkout still does not charge a card |
| **Tests** | 3 | 60% | Backend ~57 unit tests; frontend host ~15 (lib + `tests/web.rs`) in CI; Clippy backend `-D warnings`. No Mongo HTTP integration; Immich E2E operator-owned |
| **12 Factor** | 4 | 80% | Docker + env config; runtime JWT secret; pinned Rust images; `jheffmedia-*` docker tags |
| **Performance** | 3 | 60% | Actix + async Mongo; static `Cache-Control`/ETag shipped; WASM size and thumbnail load still light on verification |

**Overall weighted average:** 3.4 / 5 (68%) — **on track for personal/staging beta** per [beta-readiness.md](./beta-readiness.md); not client-production ready without further hardening and feature completion. First public beta tag remains future.

---

## Security (4 / 5)

### Criteria

| Criterion | Weight | Status | Notes |
|-----------|--------|--------|-------|
| Authentication | High | ✅ | JWT (HS256) + server-side `loginSession` in MongoDB |
| Password storage | High | ✅ | bcrypt; min 8 chars |
| Authorization | High | ⚠️ | Admin API routes enforce role. Signed-out `/admin` opens sign-in. Any other role goes to `/404`. Profile is not gated |
| Transport / edge | High | ⚠️ | HTTPS/HSTS documented for reverse proxy, not enforced in app |
| Headers | Medium | ✅ | `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` in `main.rs` |
| CORS | Medium | ✅ | Explicit origins via `YEW_FULLSTACK_CORS_ORIGINS` (not wildcard) |
| 2FA | Medium | ✅ | Enroll/confirm/disable API; login requires a valid code before creating a session |
| Abuse resistance | Medium | ⚠️ | In-memory limits: 8 auth attempts and 5 contact posts per IP per 10 minutes. Resets on process restart |
| CSP / XSS | Medium | ✅ | CSP shipped (`services/security_headers.rs`); JWT still in session storage |
| Secrets management | High | ✅ | Runtime JWT via `YEW_FULLSTACK_JWT_SECRET_FILE` or hex env; dev `secret.key` fallback |

### Evidence

- `backend/src/utils/auth.rs`, `backend/src/models/user_token.rs`
- `backend/src/services/security_headers.rs` — CSP for WASM + embeds
- `docs/authentication.md` hardening notes
- `backend/src/main.rs` middleware
- Operator: `jheffmedia --postinstall --security` · [deployment-proxmox.md § Security](./deployment-proxmox.md#security-hardening-lxc)

### To reach 4

- [x] Runtime JWT secret (env/volume mount, not compile-time embed)
- [x] Rate limiting on auth + contact endpoints
- [x] Enforce TOTP at login when `totp_enabled`
- [x] CSP tuned for WASM + embeds
- [x] Hide admin UI unless `role == admin`

### To reach 5

- [ ] Security audit / dependency scanning in CI
- [ ] Email change mailer wired before production enable
- [ ] Penetration-style auth tests
- [ ] Authenticate or network-restrict `/metrics`

---

## UX (3 / 5)

### Criteria

| Criterion | Weight | Status | Notes |
|-----------|--------|--------|-------|
| Navigation / routing | High | ✅ | 15+ Yew routes (`frontend/src/routes/mod.rs`) |
| Visual design | Medium | ✅ | Tailwind v4, shared UI components |
| Homepage / feed | High | ✅ | `MediaFeedScroll` + `/api/media/feed` |
| Auth UX | High | ✅ | Login, signup, logout, account settings |
| Shoots / galleries | High | ✅ | Admin → Content: edit/delete collections, reorder, multi-file upload, covers, featured. Public detail: admin upload/cover/remove |
| Client delivery | High | ⚠️ | Detail pages can select stills; checkout stores one size and an unpaid total. No card charge |
| Social integrations | Medium | ⚠️ | Admin → Links paste keys/tokens; empty Instagram returns 204. True OAuth out of scope |
| Contact / booking | Medium | ⚠️ | Contact emails when SMTP is set; schedule lists future Mongo dates and can still embed a calendar |
| Accessibility / mobile | Low | ⚠️ | Not formally assessed |
| Error / loading states | Medium | ⚠️ | Spinners exist; inconsistent across pages |

### Evidence

- `frontend/src/components/feed/media_feed_scroll.rs`
- `frontend/src/components/admin/collection_browser.rs` (gallery/shoot CRUD)
- `docs/TODO.md` · `frontend/src/routes/account.rs` (2FA UI)

### To reach 4

- [x] Real shoot/gallery media upload and detail pages with assets
- [x] Gallery/shoot edit, delete, reorder, and batch upload (Admin → Content)
- [ ] Immich or static feed confirmed for production homepage
- [x] Contact notification (email or webhook)
- [x] Schedule page lists owned dates; external calendar embed stays optional

### To reach 5

- [ ] Print checkout end-to-end (size and unpaid total exist; card payment shelved)
- [x] Search/filter unified feed
- [ ] Polish 2FA enrollment UX (QR display) — otpauth URL + secret is enough for now

---

## Tests (3 / 5)

### Criteria

| Criterion | Weight | Status | Notes |
|-----------|--------|--------|-------|
| Unit tests | High | ✅ | Backend ~57 (`cargo test -p jheffmedia-site-backend`). Frontend host ~15 (model + `api_errors` + `tests/web.rs` contract suite) |
| CI | High | ✅ | Trunk build, backend build, both test suites, backend Clippy in `.github/workflows/rust.yml` |
| Integration tests | High | ❌ | No HTTP handler tests against test MongoDB |
| E2E / browser | Medium | ⚠️ | `frontend/tests/web.rs` host + wasm-bindgen cases; `scripts/smoke.sh` for `/`, `/healthz`, login, feed. Immich E2E operator-owned |
| Lint / static analysis | Medium | ✅ | Backend Clippy `-D warnings` (default + `forward-frontend`) in CI; frontend WASM clippy deferred |
| Regression safety | Medium | ❌ | No API contract tests for 20+ endpoints |

### Evidence

```bash
cargo test -p jheffmedia-site-backend   # ~57 unit tests
cargo test -p jheffmedia-site-frontend  # ~15 host tests (lib + tests/web.rs)
# CI also: cargo clippy -p jheffmedia-site-backend -- -D warnings
#          (+ --features forward-frontend)
scripts/smoke.sh                        # /, /healthz, login, media/feed
```

### To reach 3

- [x] Add frontend tests to CI (host `cargo test` in `.github/workflows/rust.yml`)
- [x] Fix clippy warnings; add `cargo clippy` to CI (backend)
- [x] One smoke test per critical path (`/`, login, `media/feed`) — `scripts/smoke.sh`

### To reach 4

- [ ] Integration tests with `testcontainers` or embedded Mongo
- [x] Replace `web.rs` with route + API contract tests (DOM smoke on wasm32; full E2E still open)

### To reach 5

- [ ] E2E smoke (headless) for login + homepage feed
- [ ] Coverage threshold or critical-path checklist in CI
- [ ] Immich feed E2E in a controlled CI or documented operator gate ([beta-readiness.md](./beta-readiness.md))

---

## 12 Factor (4 / 5)

| Factor | Status | Notes |
|--------|--------|-------|
| I Codebase | ✅ | Single repo, Cargo workspace |
| II Dependencies | ✅ | Root `Cargo.lock` committed; CSS via Cargo `tailwind` (`frontend/Cargo.lock` gitignored) |
| III Config | ✅ | `YEW_FULLSTACK_*` env vars; runtime JWT secret |
| IV Backing services | ✅ | MongoDB as attachable service in Compose |
| V Build/release | ✅ | Multi-stage `Dockerfile`, optional `YEW_FULLSTACK_BUILD_ID`, `scripts/prod.sh` |
| VI Processes | ✅ | Single-replica Actix; durable upload volume documented |
| VII Port binding | ✅ | `YEW_FULLSTACK_HOST` / `PORT` |
| VIII Concurrency | ✅ | Actix async model |
| IX Disposability | ✅ | `GET /healthz` + Compose healthchecks; restart-friendly |
| X Dev/prod parity | ✅ | `scripts/dev.sh`, install targets |
| XI Logs | ✅ | `env_logger` / `RUST_LOG` |
| XII Admin processes | ✅ | Admin API, install scripts, one-off migrations manual |

### Gaps

- Legacy `YEW_FULLSTACK_*` env prefix (docker tags renamed to `jheffmedia-*`)
- Graceful shutdown / Mongo backup-restore still light on procedure detail

### To reach 4

- [x] Pin all Docker Rust images (`Dockerfile` already uses `1.85-bookworm`; align dev `backend/Dockerfile`)
- [x] Runtime JWT secret via env or mounted file
- [x] Rename docker project tags to `jheffmedia-*`

### To reach 5

- [x] Health check endpoint + Compose `healthcheck` (`GET /healthz`)
- [ ] Documented graceful shutdown and backup/restore for MongoDB

---

## Performance (3 / 5)

### Criteria

| Criterion | Weight | Status | Notes |
|-----------|--------|--------|-------|
| Server framework | High | ✅ | Actix 4 — top-tier throughput in Rust ecosystem |
| Database access | High | ✅ | MongoDB 3.x async throughout |
| Frontend delivery | Medium | ⚠️ | WASM bundle size; no code-splitting documented |
| Caching | Medium | ⚠️ | Static assets: mtime/size **ETag** + **Cache-Control** in `serve_frontend.rs` (fingerprinted immutable). Thumbnail proxy caching still open |
| External APIs | Medium | ⚠️ | Immich/IG/YT proxy; load not tested |
| Asset pipeline | Medium | ✅ | Thumbnails proxied server-side (keys not exposed) |

### Stack comparison note

Per [rust-web-framework-comparison](https://github.com/flosse/rust-web-framework-comparison), Actix and Yew remain active, stable-Rust choices. Alternatives (Axum, Leptos, Dioxus) may offer ecosystem or SSR benefits but do not inherently outperform this stack for this workload without a full rewrite.

### To reach 4

- [x] Cache-Control / ETag for static frontend assets (`serve_frontend.rs`)
- [ ] Cache-Control for thumbnail proxy
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
4. **Release gate suggestion:** average ≥ 3.5 with no category below 3 for public production; average ≥ 3 acceptable for personal/staging. Prefer the operator+maintainer checklist in [beta-readiness.md](./beta-readiness.md) before tagging `v0.1.0-beta.1`.

## Related docs

- [beta-readiness.md](./beta-readiness.md) — beta definition and exit checklist
- [TODO.md](./TODO.md) — actionable backlog
- [design.md](./design.md) — page behavior and open questions
- [architecture.md](./architecture.md) — system layout
- [deployment-proxmox.md](./deployment-proxmox.md) — production deploy · security hardening
- [authentication.md](./authentication.md) — auth security notes
