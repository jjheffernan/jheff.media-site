---
name: dev-server
description: Start, stop, and troubleshoot the jheff.media-site dev stack (Trunk + Actix + MongoDB via Docker). Use when the user asks to run dev, fix blank page, or debug localhost:8080/8000.
---

# Dev server skill

## Prerequisites

```bash
scripts/setup-dev.sh   # secret.key, wasm target, trunk
```

- `backend/src/secret.key` (32 bytes): `openssl rand -out backend/src/secret.key 32`
- Docker Desktop with Compose v2

## Start / stop

```bash
scripts/run-dev.sh     # Trunk :8000 + Actix :8080 + Mongo
scripts/stop-dev.sh
```

**Use http://localhost:8080** — Actix proxies Trunk and serves `/api/auth/*`.

## Stack

| Container | Port | Role |
|-----------|------|------|
| yew-fullstack-frontend | 8000 | `trunk serve` |
| yew-fullstack-backend | 8080 | Actix + JWT + proxy |
| yew-fullstack-database | 27017 | MongoDB 7 |

Local without Docker: `cd frontend && trunk serve --port 8000` + `cargo run -p jheffmedia-site-backend --features forward-frontend` with env from [docs/development.md](../../docs/development.md).

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Blank page, CSS only | Trunk not running or proxy broken; check `yew-fullstack-frontend.js` returns 200 on :8080 |
| `docker-compose` not found | `scripts/docker-compose.sh` |
| Backend won't compile | Missing `secret.key` |
| Workspace error in Docker | Compose mounts repo root at `/usr/src/workspace` |
| Stale containers | `scripts/stop-dev.sh` then `scripts/run-dev-force-recreate.sh` |

## Reference

[docs/deployment-dev.md](../../docs/deployment-dev.md) · [docs/development.md](../../docs/development.md)
