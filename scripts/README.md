# Scripts

Two entry points — everything else is a thin wrapper for backwards compatibility.

## Dev (`scripts/dev.sh`)

| Command | What it does |
|---------|----------------|
| `setup` | One-time: `secret.key`, wasm target, Trunk, cargo-watch, npm |
| `build` | **Local** `cargo check` + Tailwind (fast, verbose logs in `.dev/logs/`) |
| `build --docker` | Build dev Docker images (`BUILDKIT_PROGRESS=plain`) |
| `build --docker --no-cache` | Force rebuild dev images |
| `start` | **Local** Mongo in Docker + Trunk/backend/tailwind in background — **returns immediately** |
| `start --wait` | Same, then poll :8000/:8080 (optional) |
| `start --build` | Run `build` before `start` |
| `start --docker` | Full dev stack in Docker (detached) |
| `stop` | Stop local processes + dev compose stack |
| `logs [name]` | `trunk`, `backend`, `tailwind`, `docker`, `all` |
| `status` | PIDs, ports, containers |

**Recommended flow**

```bash
scripts/dev.sh setup
scripts/dev.sh build
scripts/dev.sh start          # non-blocking
scripts/dev.sh logs trunk     # watch first compile
scripts/dev.sh status
```

Runtime: `.dev/logs/`, `.dev/pids/`. Env: `scripts/config/dev.env`.

## Prod (`scripts/prod.sh`)

| Command | What it does |
|---------|----------------|
| `build` | Production Docker image (`yew-fullstack/application`) |
| `start` | `docker compose` production stack |
| `stop` | Tear down production stack |

## Legacy wrappers

| Old | Redirects to |
|-----|--------------|
| `setup-dev.sh` | `dev.sh setup` |
| `run-dev.sh` | `dev.sh start --docker` |
| `run-dev-force-recreate.sh` | `dev.sh build --docker --no-cache` + `start --docker` |
| `stop-dev.sh` | `dev.sh stop` |
| `build.sh` | `prod.sh build` |
| `run.sh` | `prod.sh start` |
| `stop.sh` | `prod.sh stop` |
| `build-and-run.sh` | `prod.sh build` + `prod.sh start` |

## Shared libraries

- `lib/common.sh` — paths, timestamped logging
- `lib/process.sh` — background PIDs, port/http waits, cleanup
- `docker-compose.sh` — `docker compose` v2 wrapper
