# Architecture

How the full stack fits together and how HTTP traffic is handled.

See also: [authentication.md](./authentication.md), [development.md](./development.md), [backend.md](./backend.md), [frontend.md](./frontend.md).

## Components

| Piece | Location | Role |
|-------|----------|------|
| Backend | `backend/` | Actix Web API, optional frontend proxy or static files |
| Frontend | `frontend/` | Yew WASM app bundled by Trunk |
| Database | MongoDB | User records and login sessions |
| Orchestration | `scripts/` | Docker Compose for dev and prod |

Cargo workspace (`Cargo.toml`): members `backend`, `frontend`.

## Dev vs production

| Mode | Backend feature | Frontend delivery | Typical command |
|------|-----------------|-------------------|-----------------|
| **Development** | `forward-frontend` | Trunk dev server; backend proxies browser traffic | `scripts/dev.sh start` |
| **Production** | default (no feature) | Pre-built static files from `YEW_FULLSTACK_STATIC` | `scripts/build.sh` + `scripts/run.sh` |

### Development path

```
Browser → :8080 Backend (Actix)
              ├─ /api/*     → API handlers (MongoDB)
              └─ /*         → proxy to :8000 Trunk (forward_frontend.rs)
```

Trunk serves WASM, JS, and static CSS. The browser usually hits **8080** so API and UI share one origin.

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

CORS allows only origins listed in `YEW_FULLSTACK_CORS_ORIGINS` (comma-separated), defaulting to `YEW_FULLSTACK_FORWARD_FRONTEND_URL` or `http://localhost:8000` — not wildcard.

## Docker Compose layouts

### Dev (`scripts/docker-compose.dev.yml`)

| Service | Container | Port |
|---------|-----------|------|
| `frontend` | `jheffmedia-frontend` | 8000 |
| `backend` | `jheffmedia-backend` | 8080 |
| `db` | `jheffmedia-database` | 27017 |
| `prometheus` | `jheffmedia-prometheus` | `127.0.0.1:9090` |
| `grafana` | `jheffmedia-grafana` | `127.0.0.1:3000` |

Backend env sets `YEW_FULLSTACK_FORWARD_FRONTEND_URL=http://frontend:8000`.

### Prod (`scripts/docker-compose.yml`)

App container and MongoDB (`mongo:7.0`). `GET /metrics` is Prometheus text for an existing Prometheus and Grafana. Prod compose uses `YEW_FULLSTACK_DB_CONNSTR` + `YEW_FULLSTACK_DB_NAME` (legacy `YEW_FULLSTACK_DB_URL` removed). Deploy guide: [deployment-dev.md](./deployment-dev.md) (dev) · version pins: [version-matrix.md](./version-matrix.md).

## Frontend application structure

```
App (layout + router)
 └── Layout (header, footer, content)
      └── Router → Home, Content, Schedule, Search, Social, Shoots, Galleries,
                   Booking, Contact, Legal, OtherSites, Account, Admin, Links, Profile, 404
Header
 └── Login + Signup (guest) OR AuthControls (logged in)
```

Route definitions: `frontend/src/routes/mod.rs` (`AppRoutes`).

Details: [frontend.md](./frontend.md).

## Data flow (auth)

Signup/login write users and sessions in MongoDB; login returns a JWT; logout clears server session. Client stores auth in session storage via `AuthEventBus`.

Full sequence: [authentication.md](./authentication.md).

## CI

GitHub Actions (`.github/workflows/rust.yml`):

- JWT secret loaded at runtime (`config/jwt.rs`); dev/tests use `backend/src/secret.key`
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
