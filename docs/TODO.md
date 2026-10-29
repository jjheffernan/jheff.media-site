# Project TODO

Track setup, quality, and feature work. **Docs:** [README.md](./README.md) · [deployment-dev.md](./deployment-dev.md)

## Dev setup

```bash
scripts/dev.sh setup
scripts/dev.sh start          # http://localhost:8080
scripts/dev.sh logs trunk     # watch WASM compile
```

- [ ] **Rust** — `stable` + `wasm32-unknown-unknown` + `trunk`
- [ ] **Node.js** ≥ 18 — Tailwind CLI only (`frontend/npm install`)
- [ ] **Docker** — MongoDB (local dev via `dev.sh` or Compose)
- [x] **JWT secret** — runtime via env/file mount; dev: `openssl rand -out backend/src/secret.key 32`

## CI / quality

- [x] Backend unit tests (25)
- [x] Frontend serde tests in `frontend/src/model/mod.rs`
- [x] GitHub Actions — Trunk build + backend with `forward-frontend`
- [x] Add `cargo test -p jheffmedia-site-frontend` to CI (host target, not wasm32)
- [x] Pin `backend/Dockerfile` Rust image (`rust:1.85-bookworm`)
- [x] Refresh stale docs still mentioning webpack (`architecture.md`, `deployment-dev.md`, `version-matrix.md`)

## Done in the product

- [x] **Content hub** — `GET /api/content/hub`
- [x] **Auth helper** — `require_auth()` / `require_admin()`
- [x] **Role-gated admin UI**
- [x] **Schedule dates** — future public Mongo events on Schedule and Booking. Admin panel can add, edit, and delete a date
- [x] **Contact** — honeypot, per-IP limit, best-effort SMTP
- [x] **Contact inbox** — admin list, status, and delete on the account page (`/api/admin/contacts`)
- [x] **Feed filter** — `instagram`, `youtube`, `gallery`, `shoot`
- [x] **Print request** — detail pages can select stills; checkout stores one size and an unpaid total
- [x] Docker tags `jheffmedia-*`

Decisions are recorded in [design.md](./design.md#decided).

## Action items

### Content

- [x] **Gallery and shoot maintenance** — edit, delete, reorder, and multi-file upload. Env JSON collections still cannot take uploads
- [x] **Schedule delete in the admin panel**
- [x] **Schedule edit** — same date fields, saved with the date id from the account admin panel
- [x] **Explicit covers and featured tag** — cover changes only when the upload asks. Featured is stored on the media item. Cards no longer invent a placeholder image. Sample env JSON may still point at picsum
- [x] **Legal pages** — titles and body use the design classes
- [x] **Homepage** — photo feed leads. Content hub is the primary call to action
- [x] **Booking request** — booking page is its own form. Contact is general questions only. Schedule copy frames dates as availability

### Account

- [x] **Remove email change** from the account page. The backend route stays
- [x] **2FA enrollment** — otpauth URL and secret are enough. No QR image
- [x] **Print payment** — shelved. Invoices or another service later. Keep the unpaid request
- [x] **Print order admin** — list every request and set `status` or `paymentStatus`. No card charge

### API client

- [ ] **Shared fetch helper** — `frontend/src/services/api.rs` is only `auth_header`. The same gloo-net parse pattern is copied across pages
- [ ] **Retire `GET /api/feed`** — homepage uses `/api/media/feed`. Remove `feed_service` after Immich thumbnails go through the unified feed, or document why the legacy route stays

### Hardening

- [ ] **Content-Security-Policy** for WASM and embeds
- [ ] **Health check** plus a Compose healthcheck
- [ ] **Cache-Control** on static files and the thumbnail proxy (`serve_frontend.rs` still has an ETag TODO)
- [ ] **Clippy** — backend warnings, then `cargo clippy` in CI
- [ ] **Browser test** — `frontend/tests/web.rs` is `1 + 1 == 2`. `scripts/smoke.sh` only checks the homepage and login reachability
- [ ] **Commit `Cargo.lock`** — it is gitignored, so builds are not pinned

### Operator setup

- [x] **Library import** — admin copies up to 6 Immich or Nextcloud files into a draft gallery or shoot. Public pages hide `draft`
- [ ] **Immich feed** — set `YEW_FULLSTACK_FEED_PROVIDER=immich` and confirm the thumbnail proxy
- [ ] **Live social** — Instagram and YouTube posts still need keys or `YEW_FULLSTACK_YOUTUBE_STATIC_JSON`
- [ ] **Proxmox path** — confirm [install/README.md](../install/README.md) matches [deployment-proxmox.md](./deployment-proxmox.md)

## Fixes applied

- 2026-06-19: MongoDB async, awc, Compose v2 wrapper
- 2026-06-20: Yew 0.23 + Trunk migration, gloo-net, CI Trunk pre_build hook
- 2026-06-22: Removed dead `PhotoFeed`/`PhotoCard` (home uses `MediaFeedScroll` + `/api/media/feed`)
