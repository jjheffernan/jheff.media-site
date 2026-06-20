# Local development

How to run and debug the stack locally.

**Doc index:** [README.md](./README.md) · deploy: [deployment-dev.md](./deployment-dev.md) · versions: [version-matrix.md](./version-matrix.md)

## Quick start

```bash
scripts/setup-dev.sh
scripts/run-dev.sh
```

Open http://localhost:8080 — Actix proxies the Yew/webpack app from port 8000.

## Stack layout

See [architecture.md](./architecture.md) for dev vs prod routing.

```
┌─────────────┐     proxy      ┌──────────────────┐
│  Browser    │ ──────────────►│ Backend (8080)   │
│             │                │  Actix + JWT API │
└─────────────┘                └────────┬─────────┘
                                        │
                    ┌───────────────────┼───────────────────┐
                    ▼                   ▼                   ▼
            ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
            │ Frontend     │    │ MongoDB      │    │ /api/auth/*  │
            │ webpack:8000 │    │ :27017       │    │ routes       │
            └──────────────┘    └──────────────┘    └──────────────┘
```

## Environment variables (dev)

Full list: [backend.md](./backend.md). Set in `scripts/docker-compose.dev.yml`:

| Variable | Dev value |
|----------|-----------|
| `YEW_FULLSTACK_HOST` | `0.0.0.0` |
| `YEW_FULLSTACK_PORT` | `8080` |
| `YEW_FULLSTACK_FORWARD_FRONTEND_URL` | `http://frontend:8000` |
| `YEW_FULLSTACK_DB_CONNSTR` | `mongodb://db:27017` |
| `YEW_FULLSTACK_DB_NAME` | `yew-fullstack_dev` |

## Local without Docker

Requires MongoDB on `mongodb://localhost:27017`.

```bash
# Frontend
cd frontend && yarn run start:dev

# Backend (separate terminal)
cd backend
export YEW_FULLSTACK_FORWARD_FRONTEND_URL=http://localhost:8000
export YEW_FULLSTACK_DB_CONNSTR=mongodb://localhost:27017
export YEW_FULLSTACK_DB_NAME=yew-fullstack_dev
cargo watch -x "run --features forward-frontend"
```

Use http://localhost:8080 for the combined app once both processes run.

## Verify

```bash
docker ps | grep yew-fullstack
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/
```

Auth smoke tests: [api.md](./api.md#examples).

## Troubleshooting

| Symptom | Likely fix |
|---------|------------|
| `docker-compose` not found | `scripts/docker-compose.sh` wraps `docker compose` v2 |
| Backend compile: `secret.key` | `openssl rand -out backend/src/secret.key 32` |
| Webpack OpenSSL error | Upgraded to webpack 5 — legacy OpenSSL flag no longer required |
| WASM build fails | `rustup target add wasm32-unknown-unknown` |
| Mongo connection refused | Wait for `db` container; check compose env |
| API 404 from :8000 | Use :8080 so `/api` hits backend |
| Stale containers | `scripts/stop-dev.sh` then `scripts/run-dev-force-recreate.sh` |

More checklist items: [TODO.md](./TODO.md).

## AI agent tooling

Cross-agent hub: **[AGENTS.md](../AGENTS.md)** · **[.agents/](../.agents/)**

| Path | Role |
|------|------|
| `.agents/rules/` | Repo conventions (project, backend, frontend) |
| `.agents/skills/` | Workflows: dev server, rust fullstack build |
| `.cursor/rules/` | caveman + ponytail (Cursor) |
| `.cursor/skills/` | Symlinks to `.agents/skills/` |

## See also

- [deployment-dev.md](./deployment-dev.md) — full dev deployment guide
- [version-matrix.md](./version-matrix.md) — dependency pins
- [TODO.md](./TODO.md) — checklist and open work
