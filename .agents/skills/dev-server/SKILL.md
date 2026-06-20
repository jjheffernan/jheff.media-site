---
name: dev-server
description: Start, stop, and troubleshoot the jheff.media-site dev stack (Trunk + Actix + MongoDB). Use when the user asks to run dev, fix blank page, or debug localhost:8080/8000.
---

# Dev server skill

## Schema

| Script | Role |
|--------|------|
| `scripts/dev.sh setup` | One-time machine setup |
| `scripts/dev.sh build` | Local compile + CSS (fast) |
| `scripts/dev.sh build --docker` | Dev Docker images (verbose, slow) |
| `scripts/dev.sh start` | **Local** dev — background processes, non-blocking |
| `scripts/dev.sh start --docker` | Full Docker dev stack (detached) |
| `scripts/dev.sh stop` | Stop local + Docker dev |
| `scripts/dev.sh logs [name]` | `trunk`, `backend`, `tailwind`, `docker`, `all` |
| `scripts/prod.sh` | Production build/start/stop |

Logs and PIDs: `.dev/logs/`, `.dev/pids/`. Config: `scripts/config/dev.env`.

## Prerequisites

```bash
scripts/dev.sh setup
```

- `backend/src/secret.key` (32 bytes)
- Docker (for MongoDB in local mode, or full stack in `--docker` mode)

## Start (recommended — local)

```bash
scripts/dev.sh build    # optional but fast sanity check
scripts/dev.sh start    # returns while Trunk compiles in background
scripts/dev.sh logs trunk
```

**Use http://localhost:8080** — Actix proxies Trunk and serves `/api/*`.

## Start (Docker — slow first compile)

```bash
scripts/dev.sh build --docker
scripts/dev.sh start --docker
scripts/dev.sh logs docker
```

## Stack

| Service | Port | Role |
|---------|------|------|
| Trunk | 8000 | Yew WASM dev server |
| Actix | 8080 | API + frontend proxy |
| MongoDB | 27017 | User/session data |

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Hangs on compile | Use local `dev.sh start`; watch `dev.sh logs trunk` |
| Blank page | Trunk still building; `dev.sh status` |
| `NO_COLOR` trunk error | Scripts unset `NO_COLOR` automatically |
| Stale processes | `scripts/dev.sh stop` |

## Reference

[docs/deployment-dev.md](../../docs/deployment-dev.md) · [docs/development.md](../../docs/development.md)
