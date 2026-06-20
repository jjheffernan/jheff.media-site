# Dev deployment

Step-by-step guide to deploy and run the **development** stack on a new machine.

Related: [development.md](./development.md) (troubleshooting) · [version-matrix.md](./version-matrix.md) (pins) · [architecture.md](./architecture.md) (routing)

## What you get

Three containers (Docker) or two local processes + MongoDB:

| Service | Port | Role |
|---------|------|------|
| Backend (Actix) | 8080 | API + proxy to frontend in dev |
| Frontend (webpack) | 8000 | Yew WASM dev server |
| MongoDB | 27017 | User + session data |

**Use http://localhost:8080** in the browser so `/api/*` and the UI share one origin.

## Prerequisites

Install before first run:

| Requirement | Verify |
|-------------|--------|
| Docker Desktop (Compose v2) | `docker compose version` |
| Node.js ≥ 18 | `node --version` |
| Yarn | `yarn --version` |
| Rust (optional for local builds) | `rustc --version` |
| OpenSSL CLI | `openssl version` |

Version pins: [version-matrix.md](./version-matrix.md).

## First-time setup

From the repository root:

```bash
scripts/setup-dev.sh
```

This script:

1. Creates `backend/src/secret.key` (32 bytes, gitignored) if missing
2. Adds `wasm32-unknown-unknown` Rust target when `rustup` is available
3. Runs `yarn install` in `frontend/`

Manual equivalent:

```bash
openssl rand -out backend/src/secret.key 32
rustup target add wasm32-unknown-unknown
cd frontend && yarn install && cd ..
```

## Deploy dev stack (Docker)

```bash
# Ensure Docker daemon is running
docker info

scripts/run-dev.sh
```

What `run-dev.sh` does:

1. `yarn install` in `frontend/`
2. `docker compose` via `scripts/docker-compose.sh` with `scripts/docker-compose.dev.yml`
3. Builds/starts `frontend`, `backend`, `db` containers
4. Exits when the backend container stops

### Container names

| Compose service | Container name |
|-----------------|----------------|
| frontend | `yew-fullstack-frontend` |
| backend | `yew-fullstack-backend` |
| db | `yew-fullstack-database` |

### Environment (dev)

Set in `scripts/docker-compose.dev.yml`:

```bash
YEW_FULLSTACK_HOST=0.0.0.0
YEW_FULLSTACK_PORT=8080
YEW_FULLSTACK_FORWARD_FRONTEND_URL=http://frontend:8000
YEW_FULLSTACK_DB_CONNSTR=mongodb://db:27017
YEW_FULLSTACK_DB_NAME=yew-fullstack_dev
```

Backend build feature: `forward-frontend` (proxy mode). See [backend.md](./backend.md).

## Verify deployment

```bash
# Containers up
docker ps --filter name=yew-fullstack

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
scripts/stop-dev.sh          # remove dev containers/images
scripts/run-dev-force-recreate.sh   # rebuild after dependency/docker issues
```

## Local deployment (no Docker)

Use when debugging Rust/Node directly. Requires MongoDB on `localhost:27017`.

**Terminal 1 — frontend**

```bash
cd frontend
yarn run start:dev
```

**Terminal 2 — backend**

```bash
cd backend
export YEW_FULLSTACK_FORWARD_FRONTEND_URL=http://localhost:8000
export YEW_FULLSTACK_DB_CONNSTR=mongodb://localhost:27017
export YEW_FULLSTACK_DB_NAME=yew-fullstack_dev_local
cargo watch -x "run --features forward-frontend"
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
| `secret.key` compile error | `scripts/setup-dev.sh` or `openssl rand ...` |
| `docker-compose` not found | Scripts use `scripts/docker-compose.sh` |
| Webpack `digital envelope` error | Upgraded to webpack 5 — legacy OpenSSL flag no longer required |
| API fails from :8000 | Use :8080 (backend proxy) |
| Mongo not ready | Wait for `db` container; check logs `docker logs yew-fullstack-database` |
| Port in use | Stop old stack: `scripts/stop-dev.sh` |

More: [development.md#troubleshooting](./development.md#troubleshooting).

## Production deployment

Prod uses a single image (`scripts/build.sh` → `scripts/run.sh`), static frontend, no webpack proxy. Env vars must use `YEW_FULLSTACK_DB_CONNSTR` + `YEW_FULLSTACK_DB_NAME` (fixed in `scripts/docker-compose.yml`). Details: [architecture.md](./architecture.md#dev-vs-production).

## Checklist

- [ ] `scripts/setup-dev.sh` completed
- [ ] Docker running
- [ ] `scripts/run-dev.sh` succeeds
- [ ] http://localhost:8080 loads
- [ ] `cargo test -p jheffmedia-site-backend` passes

Track open items: [TODO.md](./TODO.md).
