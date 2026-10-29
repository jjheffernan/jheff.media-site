# Dev deployment

Step-by-step guide to deploy and run the **development** stack on a new machine.

Related: [development.md](./development.md) (troubleshooting) · [version-matrix.md](./version-matrix.md) (pins) · [architecture.md](./architecture.md) (routing)

## What you get

Three containers (Docker) or two local processes + MongoDB:

| Service | Port | Role |
|---------|------|------|
| Backend (Actix) | 8080 | API + proxy to frontend in dev |
| Frontend (Trunk) | 8000 | Yew WASM dev server |
| MongoDB | 27017 | User + session data |

**Use http://localhost:8080** in the browser so `/api/*` and the UI share one origin.

## Prerequisites

Install before first run:

| Requirement | Verify |
|-------------|--------|
| Docker Desktop (Compose v2) | `docker compose version` |
| Node.js ≥ 18 | `node --version` |
| npm | 9+ | `npm --version` |
| Trunk | latest | `trunk --version` (installed by setup) |
| Rust (optional for local builds) | `rustc --version` |
| OpenSSL CLI | `openssl version` |

Version pins: [version-matrix.md](./version-matrix.md).

## First-time setup

From the repository root:

```bash
scripts/dev.sh setup
```

This script:

1. Creates `backend/src/secret.key` (32 bytes, gitignored) if missing
2. Adds `wasm32-unknown-unknown` Rust target when `rustup` is available
3. Installs Trunk and `cargo-watch` when missing
4. Runs `npm install` + `npm run css:build` in `frontend/`

Manual equivalent:

```bash
openssl rand -out backend/src/secret.key 32
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
cd frontend && npm install && npm run css:build && cd ..
```

## Deploy dev stack (Docker)

```bash
# Ensure Docker daemon is running
docker info

scripts/dev.sh start --docker
```

What `scripts/dev.sh start --docker` does:

1. `npm install` in `frontend/` (via Docker build)
2. `docker compose` via `scripts/docker-compose.sh` with `scripts/docker-compose.dev.yml`
3. Builds/starts `frontend`, `backend`, `db` containers
4. Detaches — use `scripts/dev.sh logs docker` to tail

### Container names

| Compose service | Container name |
|-----------------|----------------|
| frontend | `jheffmedia-frontend` |
| backend | `jheffmedia-backend` |
| db | `jheffmedia-database` |

### Environment (dev)

Set in `scripts/docker-compose.dev.yml`:

```bash
YEW_FULLSTACK_HOST=0.0.0.0
YEW_FULLSTACK_PORT=8080
YEW_FULLSTACK_FORWARD_FRONTEND_URL=http://frontend:8000
YEW_FULLSTACK_DB_CONNSTR=mongodb://db:27017
YEW_FULLSTACK_DB_NAME=jheffmedia_dev
```

Backend build feature: `forward-frontend` (proxy mode). See [backend.md](./backend.md).

## Verify deployment

```bash
# Containers up
docker ps --filter name=jheffmedia

# HTTP
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/

# Signup + login (API)
curl -s -X POST http://localhost:8080/api/auth/signup \
  -H 'Content-Type: application/json' \
  -d '{"email":"dev@example.com","username":"devuser","password":"devpass"}'

curl -s -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"emailOrUsername":"devuser","password":"devpass"}'
```

Open http://localhost:8080 — header should show login/signup forms; after login, greeting + logout.

Full API reference: [api.md](./api.md).

## Stop and reset

```bash
scripts/dev.sh stop          # stop local processes + dev containers
scripts/dev.sh start --docker --build   # rebuild after dependency/docker issues
```

## Local deployment (no Docker)

Use when debugging Rust/Node directly. Requires MongoDB on `localhost:27017`.

Preferred: `scripts/dev.sh start` (from repo root).

**Manual — Terminal 1 — CSS watch**

```bash
cd frontend && npm run css:watch
```

**Terminal 2 — Trunk**

```bash
cd frontend && trunk serve --port 8000 --address 0.0.0.0
```

**Terminal 3 — backend**

```bash
cd backend
export YEW_FULLSTACK_FORWARD_FRONTEND_URL=http://localhost:8000
export YEW_FULLSTACK_DB_CONNSTR=mongodb://localhost:27017
export YEW_FULLSTACK_DB_NAME=jheffmedia_dev_local
cargo watch -x "run -p jheffmedia-site-backend --features forward-frontend"
```

Browse http://localhost:8080.

## Run tests before/after deploy

```bash
openssl rand -out backend/src/secret.key 32   # if missing

cargo test -p jheffmedia-site-backend
cargo test -p jheffmedia-site-frontend
```

CI runs the same backend tests on push. See `.github/workflows/rust.yml`.

## Common deployment failures

| Symptom | Fix |
|---------|-----|
| `secret.key` compile error | `scripts/dev.sh setup` or `openssl rand ...` |
| `docker-compose` not found | Scripts use `scripts/docker-compose.sh` |
| Trunk not found | `scripts/dev.sh setup` or `cargo install --locked trunk` |
| API fails from :8000 | Use :8080 (backend proxy) |
| Mongo not ready | Wait for `db` container; check logs `docker logs jheffmedia-database` |
| Port in use | Stop old stack: `scripts/dev.sh stop` |

More: [development.md#troubleshooting](./development.md#troubleshooting).

## Production deployment

Prod uses a single image (`scripts/build.sh` → `scripts/run.sh`), static frontend, no Trunk proxy. Env vars must use `YEW_FULLSTACK_DB_CONNSTR` + `YEW_FULLSTACK_DB_NAME` (fixed in `scripts/docker-compose.yml`). Details: [architecture.md](./architecture.md#dev-vs-production).

## Checklist

- [ ] `scripts/dev.sh setup` completed
- [ ] Docker running
- [ ] `scripts/dev.sh start --docker` succeeds
- [ ] http://localhost:8080 loads
- [ ] `cargo test -p jheffmedia-site-backend` passes

Track open items: [TODO.md](./TODO.md).
