# Project TODO

Track quality and feature work. **Docs:** [README.md](./README.md) · [deployment-dev.md](./deployment-dev.md) · [readiness-rubric.md](./readiness-rubric.md) · [beta-readiness.md](./beta-readiness.md)

Local stack: `scripts/dev.sh setup && scripts/dev.sh start` → http://localhost:8080. Setup installs Rust target, Trunk, and Cargo `tailwind` when missing — no separate checklist noise.

## Done in the product

- [x] Content hub, role-gated admin, schedule CRUD, contact honeypot/limits/SMTP + admin inbox
- [x] Feed filter (`instagram`, `youtube`, `gallery`, `shoot`); homepage via `/api/media/feed`
- [x] Gallery/shoot maintenance — Admin → Content: edit, delete, reorder, multi-file upload, covers, featured. Env JSON collections still cannot take uploads
- [x] Public gallery/shoot detail: admin can upload / set cover / remove (reorder + featured checkbox live under Admin → Content)
- [x] Legal pages, homepage feed-first, booking vs contact split
- [x] Account: no email-change UI; 2FA otpauth+secret (no QR); print orders unpaid + admin status only
- [x] Library import (≤6 Immich/Nextcloud → draft); live social paste-keys in Admin → Links
- [x] Deploy channels `main` / `testing`; `jheffmedia --update` / `-d`; Admin → Dev deploy trigger + autoupdate **status**
- [x] Yew UI kit vertical slices + design tokens; CSS via Cargo `tailwind` (no Node/npm)
- [x] Booking preferred-time UX (compact typeable line) — on current `dev` tree (`d301791`); not necessarily on `main` yet
- [x] Links analytics; Docker tags `jheffmedia-*`; healthz; CI Trunk + both crate tests; root `Cargo.lock` tracked
- [x] Docs: webpack/Node leftovers cleared; Proxmox install path reviewed 2026-11-15 (see Operator setup)

Decisions: [design.md](./design.md#decided).

## Open engineering

### API client

- [x] **Shared fetch helper** — `frontend/src/services/api.rs` (`get`/`post`/`put`/`patch`/`delete` + auth + envelope errors); ~35 call sites migrated
- [x] **Retire `GET /api/feed`** — Immich/static/Nextcloud items merge into `/api/media/feed`; thumbnails at `/api/media/thumbnail/{id}`; `feed_service` removed

### Hardening / quality

- [x] **Content-Security-Policy** for WASM and embeds
- [x] **Cache-Control / ETag** — `serve_frontend.rs` (mtime/size ETag; fingerprinted assets immutable)
- [x] **Clippy** — backend clean under `-D warnings` (default + `forward-frontend`); CI runs both; frontend WASM clippy deferred
- [x] **Real browser tests** — `frontend/tests/web.rs` covers feed/login/contact serde, `AppRoutes`, theme prefs, `api_errors` scrubbing, DOM smoke (wasm32). Host CI runs the same via `cargo test`. `scripts/smoke.sh` still covers `/`, `/healthz`, login reachability
- [ ] **Immich feed E2E** — **needs live Immich** (do not commit host/key/album). **Primary:** Admin → Setup → provider Immich → paste URL / API key / album UUID → Save → **Test Immich** (hits `/api/media/feed?source=immich` + a thumbnail). **Alt (bootstrap/smoke):** `cp scripts/config/dev.env.local.example scripts/config/dev.env.local`, fill values (gitignored), `scripts/dev.sh start`, then `scripts/smoke.sh` (skips when unset; fails if env demands Immich but feed/thumbs break). Mongo `site_config` overrides env after first Setup save. See [deployment-dev.md](./deployment-dev.md#immich-feed-local)

## Deferred / out of scope

- Social **OAuth** — paste keys/tokens only; true OAuth stays out
- GUI **autoupdate enable/disable** — Admin shows status; enable/disable stays CLI (`jheffmedia --autoupdate …`)
- Media **reorder / featured** controls on the **public** gallery detail page — use Admin → Content
- **Print payment** — shelved; keep unpaid requests + admin status
- CLI **getopts** combined shorts / full rewrite
- Env JSON collections **cannot** take uploads (by design for now)

## Operator / docs verification

- [x] **Deploy starting guide** — [deployment.md](./deployment.md) + [install/registry.json](../install/registry.json)
- [x] **Proxmox path** — reviewed 2026-11-15: one-liner + `install/proxmox/*` match Core/`var_version=13`; `deployment-proxmox.md` aligned (env path, day-2 `jheffmedia --update`, feed verify notes). Manual compose via `install/common/` remains valid fallback
- [x] **Setup hygiene** — demoted local Rust/Trunk/Docker checkbox noise; `dev.sh setup` is the path
- [ ] **Beta readiness** — operator+maintainer checklist in [beta-readiness.md](./beta-readiness.md); after install run `jheffmedia --postinstall` and `jheffmedia --postinstall --security` ([Security hardening](./deployment-proxmox.md#security-hardening-lxc)). Public tag `v0.1.0-beta.1` only after checklist green

## Fixes applied (historical)

- 2026-06-19: MongoDB async, awc, Compose v2 wrapper
- 2026-06-20: Yew 0.23 + Trunk migration, gloo-net, CI Trunk pre_build hook
- 2026-06-22: Removed dead `PhotoFeed`/`PhotoCard` (home uses `MediaFeedScroll` + `/api/media/feed`)
- 2026-11-15: Docs hygiene — TODO truth, readiness rubric galleries/CRUD, Proxmox/deploy stale claims
- 2026-11-15: Engineering backlog — shared fetch helper, retire `/api/feed`, CSP+ETag, Clippy CI, real frontend tests; Immich E2E still needs live Immich
- 2026-11-15: Immich ingest path — Admin Setup polish + Test Immich; gitignored `dev.env.local`; E2E still unchecked until live verify
- 2026-11-15: Beta readiness doc + rubric bump (CSP/Clippy/tests); security postinstall pointer under Operator
