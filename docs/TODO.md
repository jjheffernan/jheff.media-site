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

- [x] Backend unit tests (9)
- [x] Frontend serde tests in `frontend/src/model/mod.rs` (2)
- [x] GitHub Actions — Trunk build + backend with `forward-frontend`
- [ ] Add `cargo test -p jheffmedia-site-frontend` to CI (host target, not wasm32)
- [ ] Fix backend clippy warnings (12 — unused fields, redundant `Error::from`)
- [x] Pin `backend/Dockerfile` Rust image (`rust:1.85-bookworm`)
- [x] Refresh stale docs still mentioning webpack (`architecture.md`, `deployment-dev.md`, `version-matrix.md`)

## Frontend ↔ backend split

**Healthy today:** frontend = UI + fetch; backend = auth, MongoDB, external APIs (Immich/Instagram/YouTube), feed merge, thumbnails.

- [ ] **Shared API client** — thin `frontend/src/services/api.rs` wrapper around gloo-net (auth header, JSON parse, error map); remove copy-paste in ~15 components
- [ ] **Content hub bundle** — optional `GET /api/content/hub` returning social + shoots + galleries slices (ContentHub currently fires 3 requests)
- [ ] **Deprecate `/api/feed`** — legacy Immich/static grid; home uses `/api/media/feed`. Keep Immich via unified feed or document env migration, then remove `feed_service` when unused
- [ ] **Auth middleware helper** — extract `require_auth()` / `require_admin()` in backend to DRY controllers (account, print, admin)
- [ ] **Role-gated UI** — hide admin panel client-side when `role != admin` (backend already enforces on `/api/admin/*`)

## Site functionality

- [ ] **Shoots / galleries** — image upload + public detail display (admin upload UI + `/api/content/media`); still missing edit/delete, multi-file batch UX
- [ ] **Print selection UX** — persist works; add gallery/shoot context and checkout flow
- [ ] **Schedule page** — wire external calendar URL from `/api/schedule` (config-only today)
- [ ] **Contact form** — backend stores/sends; add rate limit + honeypot
- [ ] **2FA enrollment UI** — API exists on `/api/auth/2fa/*`; polish QR/display on account page
- [ ] **Immich production feed** — set `YEW_FULLSTACK_FEED_PROVIDER=immich` + env vars; verify thumbnail proxy under load
- [ ] **YouTube/Instagram live** — replace demo fallbacks when API keys configured
- [ ] **E2E smoke test** — `scripts/smoke.sh` covers homepage + login; extend `frontend/tests/web.rs` for route render
- [ ] **Search / filter** — unified feed filter by source (instagram, youtube, gallery, shoot)

## Infra / naming

- [x] Rename legacy docker project tags (`yew-fullstack-dev` → `jheffmedia-dev`)
- [ ] Proxmox install path — finish [install/](../install/README.md) docs alignment

## Fixes applied

- 2026-06-19: MongoDB async, awc, Compose v2 wrapper
- 2026-06-20: Yew 0.23 + Trunk migration, gloo-net, CI Trunk pre_build hook
- 2026-06-22: Removed dead `PhotoFeed`/`PhotoCard` (home uses `MediaFeedScroll` + `/api/media/feed`)
