# Local development

How to run and debug the stack locally.

**Doc index:** [README.md](./README.md) · deploy: [deployment-dev.md](./deployment-dev.md) · versions: [version-matrix.md](./version-matrix.md)

## Quick start

```bash
scripts/dev.sh setup
scripts/dev.sh build
scripts/dev.sh start
```

Open http://localhost:8080 — Actix proxies the Trunk app from port 8000.

See [scripts/README.md](../scripts/README.md) for the full dev/prod command schema.

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
            │ Trunk :8000  │    │ :27017       │    │ routes       │
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
| `YEW_FULLSTACK_DB_NAME` | `jheffmedia_dev` |

## Local without Docker

Requires MongoDB on `mongodb://localhost:27017`.

Preferred: `scripts/dev.sh start` (starts Trunk, Tailwind watch, backend, and MongoDB).

Manual (separate terminals):

```bash
# Frontend + CSS
cd frontend && npm install && npm run css:watch   # terminal 1
cd frontend && trunk serve --port 8000 --address 0.0.0.0   # terminal 2

# Backend
cd backend
export YEW_FULLSTACK_FORWARD_FRONTEND_URL=http://localhost:8000
export YEW_FULLSTACK_DB_CONNSTR=mongodb://localhost:27017
export YEW_FULLSTACK_DB_NAME=jheffmedia_dev
cargo watch -x "run -p jheffmedia-site-backend --features forward-frontend"
```

Use http://localhost:8080 for the combined app once both processes run.

## Verify

```bash
docker ps | grep jheffmedia
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/
```

Auth smoke tests: [api.md](./api.md#examples).

## Troubleshooting

| Symptom | Likely fix |
|---------|------------|
| `docker-compose` not found | `scripts/docker-compose.sh` wraps `docker compose` v2 |
| Backend exits: JWT secret missing | `scripts/dev.sh setup` or `openssl rand -out backend/src/secret.key 32` |
| Trunk not found | `scripts/dev.sh setup` (installs Trunk + wasm target) |
| WASM build fails | `rustup target add wasm32-unknown-unknown` |
| Mongo connection refused | Wait for `db` container; check compose env |
| API 404 from :8000 | Use :8080 so `/api` hits backend |
| Stale containers | `scripts/dev.sh stop` then `scripts/dev.sh start --docker --build` |

More checklist items: [TODO.md](./TODO.md).

## AI agent tooling

Cross-agent hub: **[AGENTS.md](../AGENTS.md)** · **[.agents/](../.agents/)** · **[.agents/cursor.md](../.agents/cursor.md)** (Cursor)

| Path | Role |
|------|------|
| `.agents/rules/` | Repo conventions (project, backend, frontend) |
| `.agents/skills/` | Workflows and slash skills (`ponytail`, `caveman`, …) |
| `.cursor/skills/` | Symlinks only — run `.agents/scripts/sync-cursor-skills.sh` after adding a skill |

## See also

- [deployment-dev.md](./deployment-dev.md) — full dev deployment guide
- [version-matrix.md](./version-matrix.md) — dependency pins
- [TODO.md](./TODO.md) — checklist and open work
