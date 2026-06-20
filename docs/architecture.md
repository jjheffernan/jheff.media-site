# Architecture

How the full stack fits together and how HTTP traffic is handled.

See also: [authentication.md](./authentication.md), [development.md](./development.md), [backend.md](./backend.md), [frontend.md](./frontend.md).

## Components

| Piece | Location | Role |
|-------|----------|------|
| Backend | `backend/` | Actix Web API, optional frontend proxy or static files |
| Frontend | `frontend/` | Yew WASM app bundled by webpack |
| Database | MongoDB | User records and login sessions |
| Orchestration | `scripts/` | Docker Compose for dev and prod |

Cargo workspace (`Cargo.toml`): members `backend`, `frontend`.

## Dev vs production

| Mode | Backend feature | Frontend delivery | Typical command |
|------|-----------------|-------------------|-----------------|
| **Development** | `forward-frontend` | Webpack dev server; backend proxies browser traffic | `scripts/run-dev.sh` |
| **Production** | default (no feature) | Pre-built static files from `YEW_FULLSTACK_STATIC` | `scripts/build.sh` + `scripts/run.sh` |

### Development path

```
Browser → :8080 Backend (Actix)
              ├─ /api/*     → API handlers (MongoDB)
              └─ /*         → proxy to :8000 webpack (forward_frontend.rs)
```

Webpack serves WASM, JS, and static CSS. The browser usually hits **8080** so API and UI share one origin.

Direct access to **:8000** still works for frontend-only debugging; API calls from there go to the same host path `/api/...` on port 8000 and will fail unless you proxy separately—prefer **8080** in dev.

### Production path

```
Browser → :8080 Backend
              ├─ /api/*     → API handlers
              └─ /*         → files from YEW_FULLSTACK_STATIC (serve_frontend.rs)
```

Production image builds the frontend into the static directory inside the container (see root `Dockerfile` and `scripts/build.sh`).

## Request routing (backend)

Registered in `backend/src/config/app.rs`:

1. **`/api/auth/*`** — signup, login, logout ([api.md](./api.md))
2. **Everything else** — either reverse proxy (dev) or static file server (prod)

CORS is permissive (`send_wildcard`) for browser API calls from the same app origin.

## Docker Compose layouts

### Dev (`scripts/docker-compose.dev.yml`)

Three services:

| Service | Container | Port |
|---------|-----------|------|
| `frontend` | `yew-fullstack-frontend` | 8000 |
| `backend` | `yew-fullstack-backend` | 8080 |
| `db` | `yew-fullstack-database` | 27017 |

Backend env sets `YEW_FULLSTACK_FORWARD_FRONTEND_URL=http://frontend:8000`.

### Prod (`scripts/docker-compose.yml`)

Single app container + MongoDB (`mongo:7.0`). Prod compose uses `YEW_FULLSTACK_DB_CONNSTR` + `YEW_FULLSTACK_DB_NAME` (legacy `YEW_FULLSTACK_DB_URL` removed). Deploy guide: [deployment-dev.md](./deployment-dev.md) (dev) · version pins: [version-matrix.md](./version-matrix.md).

## Frontend application structure

```
App (layout + router)
 └── Layout (header, footer, content)
      └── Router → Home | Profile | 404
Header
 └── Login + Signup (guest) OR AuthControls (logged in)
```

Details: [frontend.md](./frontend.md).

## Data flow (auth)

Signup/login write users and sessions in MongoDB; login returns a JWT; logout clears server session. Client stores auth in session storage via `AuthEventBus`.

Full sequence: [authentication.md](./authentication.md).

## CI

GitHub Actions (`.github/workflows/rust.yml`):

- Generates ephemeral `secret.key` for compile
- Builds backend with `forward-frontend`
- Builds frontend for `wasm32-unknown-unknown`
- Runs backend tests

## Related files

| Concern | File |
|---------|------|
| Route registration | `backend/src/config/app.rs` |
| Dev proxy | `backend/src/services/forward_frontend.rs` |
| Static serving | `backend/src/services/serve_frontend.rs` |
| Compose dev | `scripts/docker-compose.dev.yml` |
| Compose prod | `scripts/docker-compose.yml` |
