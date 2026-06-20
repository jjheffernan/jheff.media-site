# Version matrix

Pinned and resolved dependency versions for jheff.media-site. Regenerate lockfile after changing pins: `cargo update` (intentional bumps only).

See also: [deployment-dev.md](./deployment-dev.md), [architecture.md](./architecture.md).

## Runtime requirements

| Tool | Minimum | Notes |
|------|---------|-------|
| Rust | 1.85+ (stable) | `rustup update stable` |
| Node.js | 18+ | 22 OK with webpack 5 (no OpenSSL legacy flag) |
| Yarn | 1.22+ | via `corepack enable` |
| Docker | 24+ with Compose v2 | `docker compose version` |
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
| webpack | ^5.99 | No `NODE_OPTIONS=--openssl-legacy-provider` |
| webpack-dev-server | ^5.2 | Dev server on :8000 |
| @wasm-tool/wasm-pack-plugin | ^1.7 | Invokes wasm-pack during webpack build |
| copy-webpack-plugin | ^12 | Static asset copy to `dist/` |

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
| `YEW_FULLSTACK_DB_CONNSTR` | yes | `mongodb://db:27017` | `mongodb://root:password@db:27017/yew-fullstack?authSource=admin` |
| `YEW_FULLSTACK_DB_NAME` | yes | `yew-fullstack_dev` | `yew-fullstack` |
| `YEW_FULLSTACK_DB_URL` | **no** | — | removed (legacy) |
| `YEW_FULLSTACK_DB_USER` | **no** | — | removed (legacy) |
| `YEW_FULLSTACK_DB_PASSWORD` | **no** | — | removed (legacy) |

## Legacy naming (boilerplate)

Still present in scripts/images; safe to rename in a dedicated cleanup:

| Legacy | Current crate/binary |
|--------|---------------------|
| `yew-fullstack-backend` (old Dockerfile) | `jheffmedia-site-backend` |
| `yew-fullstack/application` (docker tag) | prod image name in `scripts/build.sh` |
| `YEW_FULLSTACK_*` env prefix | unchanged for compatibility |

## CI (`.github/workflows/rust.yml`)

- Generates ephemeral `secret.key` per run
- Backend: `--features forward-frontend`
- Frontend: `wasm32-unknown-unknown` build
- Tests: `cargo test -p jheffmedia-site-backend -p jheffmedia-site-frontend`

## When you change versions

1. Update pin in `Cargo.toml` / `package.json`
2. Run `cargo build` / `yarn install` and fix compile errors
3. Update this file and [TODO.md](./TODO.md) if constraints change
4. Run full test suite: `cargo test --workspace`
