# Version matrix

Pinned and resolved dependency versions for jheff.media-site. Regenerate lockfile after changing pins: `cargo update` (intentional bumps only).

See also: [deployment-dev.md](./deployment-dev.md), [architecture.md](./architecture.md).

## Runtime requirements

| Tool | Minimum | Notes |
|------|---------|-------|
| Rust | 1.85+ (stable) | `rustup update stable` |
| Node.js | 18+ | Tailwind CLI via npm |
| npm | 9+ | `npm install` in `frontend/` |
| Docker or Podman | Compose v2 (`docker compose` or `podman compose`) | `CONTAINER_RUNTIME` picks one |
| MongoDB (image) | 7.0 | Pinned in compose files |

## Rust workspace

| Crate | Pin (`Cargo.toml`) | Role |
|-------|-------------------|------|
| `jheffmedia-site-backend` | — | Actix API |
| `jheffmedia-site-frontend` | — | Yew WASM |

Shared workspace deps (`Cargo.toml` `[workspace.dependencies]`): `serde`, `serde_json`, `log`, `uuid`, `wasm-bindgen`.

### Backend (`backend/Cargo.toml`)

| Dependency | Pin | Notes |
|------------|-----|-------|
| actix-web | 4.13 | Async HTTP server |
| actix-cors | 0.7.1 | `Cors::default()` — no `.finish()` |
| mongodb | 3.7 | Driver 3.x is async-only |
| bson | 2.15 | Matches mongodb 3.7 |
| awc | 3.8 | Replaces removed `actix_web::client` |
| jsonwebtoken | 9.3.1 | HS256 JWT |
| bcrypt | 0.15.1 | Password hashing |

### Frontend (`frontend/Cargo.toml`)

| Dependency | Pin | Notes |
|------------|-----|-------|
| yew | 0.23 | `csr` feature; `Renderer`, hooks, `ContextProvider` |
| yew-router | 0.20 | `Routable` derive, `BrowserRouter`, `Switch` |
| gloo-net | 0.6 | HTTP from WASM (`Request::post`, `send`) |
| gloo-storage | 0.3 | Session storage for auth JSON |
| wasm-bindgen-futures | 0.4.75 | `spawn_local` for async HTTP |
| console_log | 1.0.0 | Replaces `web_logger` / stdweb |

**Removed / incompatible:** `css-in-rust`, `web_logger`, Yew 0.16 agents / `FetchService`.

## JavaScript (`frontend/package.json`)

| Package | Range | Notes |
|---------|-------|-------|
| tailwindcss | ^4.3 | Utility CSS framework |
| @tailwindcss/cli | ^4.3 | `npm run css:build` / `css:watch` |

Bundling: [Trunk](https://trunk-rs.github.io/trunk/) (`frontend/Trunk.toml`), not webpack. Trunk runs `npm run css:build` as a pre-build hook.

## Docker images

| Image | File | Pin | Issue fixed |
|-------|------|-----|-------------|
| Frontend dev | `frontend/Dockerfile` | `node:20-bookworm` | Was `node:latest` |
| Backend dev | `backend/Dockerfile` | `rust:latest` | Consider pinning to `rust:1.85-bookworm` |
| Prod app | `Dockerfile` | `rust:1.85-bookworm` + `debian:bookworm-slim` | Binary `jheffmedia-site-backend` |
| MongoDB | compose files | `mongo:7.0` | Was unpinned `mongo` |

## Environment variable alignment

| Variable | Read by backend? | Dev compose | Prod compose (fixed) |
|----------|------------------|-------------|----------------------|
| `YEW_FULLSTACK_DB_CONNSTR` | yes | `mongodb://db:27017` | `mongodb://root:password@db:27017/jheffmedia?authSource=admin` |
| `YEW_FULLSTACK_DB_NAME` | yes | `jheffmedia_dev` | `jheffmedia` |
| `YEW_FULLSTACK_DB_URL` | **no** | — | removed (legacy) |
| `YEW_FULLSTACK_DB_USER` | **no** | — | removed (legacy) |
| `YEW_FULLSTACK_DB_PASSWORD` | **no** | — | removed (legacy) |

## Legacy naming (boilerplate)

Still present in scripts/images; safe to rename in a dedicated cleanup:

| Legacy | Current crate/binary |
|--------|---------------------|
| `jheffmedia-backend` (dev container) | `jheffmedia-site-backend` (binary) |
| `jheffmedia/application` (docker tag) | prod image in `scripts/prod.sh` |
| `YEW_FULLSTACK_*` env prefix | unchanged for compatibility |

## CI (`.github/workflows/rust.yml`)

- Generates ephemeral `secret.key` per run
- Backend: `--features forward-frontend`
- Frontend: `wasm32-unknown-unknown` build
- Tests: `cargo test -p jheffmedia-site-backend -p jheffmedia-site-frontend`

## When you change versions

1. Update pin in `Cargo.toml` / `package.json`
2. Run `cargo build` / `npm install` and fix compile errors
3. Update this file and [TODO.md](./TODO.md) if constraints change
4. Run full test suite: `cargo test --workspace`
