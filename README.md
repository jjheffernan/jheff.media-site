# jheff.media-site

Landing page to showcase automotive socials — built with **Yew** (WASM frontend) and **Actix Web** (Rust backend).

## Stack

| Layer | Tech |
|-------|------|
| Frontend | [Yew](https://yew.rs/) 0.23 + [Trunk](https://trunk-rs.github.io/trunk/), automotive photography UI |
| Backend | [Actix Web](https://actix.rs/) 4, JWT auth |
| Database | MongoDB |

## Prerequisites

- [Rust](https://rustup.rs/) (`stable` + `wasm32-unknown-unknown`)
- [Trunk](https://trunk-rs.github.io/trunk/) (`cargo install --locked trunk`)
- [Docker](https://www.docker.com/) or [Podman](https://podman.io/) with Compose (`docker compose` or `podman compose`). Set `CONTAINER_RUNTIME=podman` to force Podman when both are installed.

## Quick start

```bash
scripts/dev.sh setup              # once: secret, trunk, npm, wasm target
scripts/dev.sh build              # local compile check (fast)
scripts/dev.sh start              # local dev — returns while Trunk compiles in background
scripts/dev.sh logs trunk         # watch compilation progress
scripts/dev.sh status             # ports, PIDs, containers
```

| Service | URL |
|---------|-----|
| App (via backend proxy) | http://localhost:8080 |
| Frontend dev server | http://localhost:8000 |
| MongoDB | localhost:27017 |

Docker dev (slow first build): `scripts/dev.sh build --docker && scripts/dev.sh start --docker`

Stop: `scripts/dev.sh stop`

Full setup, env vars, and troubleshooting: [docs/README.md](docs/README.md) · [docs/development.md](docs/development.md) · [docs/TODO.md](docs/TODO.md)

## Documentation

| Guide | Topics |
|-------|--------|
| [docs/README.md](docs/README.md) | Index of all docs |
| [docs/architecture.md](docs/architecture.md) | Request routing, dev vs prod, Docker layout |
| [docs/authentication.md](docs/authentication.md) | JWT, sessions, login/logout behavior |
| [docs/api.md](docs/api.md) | REST endpoints and examples |
| [docs/backend.md](docs/backend.md) | Actix, env vars, MongoDB |
| [docs/deployment-dev.md](docs/deployment-dev.md) | **Dev deployment** — first-time setup, Docker, verify |
| [docs/version-matrix.md](docs/version-matrix.md) | Dependency pins and mismatch fixes |

## Scripts

| Command | Purpose |
|---------|---------|
| `scripts/dev.sh setup` | One-time setup (secret, wasm, trunk, npm) |
| `scripts/dev.sh build` | Local compile check + CSS (fast) |
| `scripts/dev.sh build --docker` | Build dev Docker images (verbose, slow) |
| `scripts/dev.sh start` | Local dev — Trunk + backend in background |
| `scripts/dev.sh start --docker` | Full dev stack in Docker (detached) |
| `scripts/dev.sh stop` | Stop local processes + dev containers |
| `scripts/dev.sh logs [name]` | Tail logs: trunk, backend, tailwind, docker, all |
| `scripts/prod.sh build` | Production Docker image |
| `scripts/prod.sh start` | Run production stack |
| `scripts/prod.sh stop` | Tear down production |

Legacy wrappers (`setup-dev.sh`, `run-dev.sh`, `stop-dev.sh`, `build.sh`, etc.) forward to the commands above.
| `scripts/stop.sh` | Stop production containers |

## Project layout

```
backend/     Actix API, MongoDB models, JWT auth
frontend/    Yew WASM app, Trunk bundler, automotive photography routes
scripts/     Docker Compose + dev orchestration
docs/        Human docs — start at docs/README.md
.agents/     Agent rules and skills (see AGENTS.md)
```

## AI agent rules

Cross-agent instructions: **[AGENTS.md](AGENTS.md)** and **[.agents/](.agents/)** (canonical). Cursor shim: **[.agents/cursor.md](.agents/cursor.md)**.

| Location | Purpose |
|----------|---------|
| `AGENTS.md` | Entry index — links to `.agents/`, not duplicate content |
| `.agents/rules/` | Project, backend, frontend conventions |
| `.agents/skills/` | Workflows + `/ponytail`, `/caveman` slash skills |
| `.cursor/skills/` | Symlinks into `.agents/skills/` (Cursor requirement) |

Community skills (adapted from upstream):

- [caveman](https://github.com/JuliusBrussee/caveman) — `.agents/skills/caveman/`
- [ponytail](https://github.com/DietrichGebert/ponytail) — `.agents/skills/ponytail/`

## About

Forked from the Yew Fullstack boilerplate. Authentication uses JWT with session validation in MongoDB.

Customize project naming: search/replace `yew-fullstack`, `YEW_FULLSTACK`, `Yew Fullstack Boilerplate`, and `Yew Fullstack`.

## License

MIT / Apache-2.0 (see `LICENSE_MIT`, `LICENSE_APACHE`).
