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
- [Docker Desktop](https://www.docker.com/) (Compose v2 — `docker compose`)

## Quick start

```bash
# JWT signing key (32 bytes, not committed — required before first backend build)
scripts/setup-dev.sh          # secret key, wasm target, trunk

scripts/run-dev.sh
```

| Service | URL |
|---------|-----|
| App (via backend proxy) | http://localhost:8080 |
| Frontend dev server | http://localhost:8000 |
| MongoDB | localhost:27017 |

Stop: `scripts/stop-dev.sh`

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
| `scripts/setup-dev.sh` | First-time setup (secret key, wasm target, yarn) |
| `scripts/run-dev.sh` | Dev stack (frontend + backend + MongoDB) |
| `scripts/stop-dev.sh` | Tear down dev containers |
| `scripts/run-dev-force-recreate.sh` | Rebuild images after dependency/cache issues |
| `scripts/build.sh` | Production Docker image |
| `scripts/run.sh` | Run production container |
| `scripts/build-and-run.sh` | Build + run production |
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

Cross-agent instructions: **[AGENTS.md](AGENTS.md)** and **[.agents/](.agents/)**

| Location | Purpose |
|----------|---------|
| `AGENTS.md` | Entry point for Codex, Antigravity, Copilot CLI, etc. |
| `.agents/rules/` | Project, backend, frontend conventions |
| `.agents/skills/` | Task workflows (`dev-server`, `rust-fullstack`) |
| `.cursor/rules/` | Cursor always-on rules (caveman, ponytail) |
| `.cursor/skills/` | Symlinks to `.agents/skills/` for Cursor skill picker |

Community rules also installed:

- [caveman](https://github.com/JuliusBrussee/caveman) — `.cursor/rules/caveman.mdc`
- [ponytail](https://github.com/DietrichGebert/ponytail) — `.cursor/rules/ponytail.mdc`

## About

Forked from the Yew Fullstack boilerplate. Authentication uses JWT with session validation in MongoDB.

Customize project naming: search/replace `yew-fullstack`, `YEW_FULLSTACK`, `Yew Fullstack Boilerplate`, and `Yew Fullstack`.

## License

MIT / Apache-2.0 (see `LICENSE_MIT`, `LICENSE_APACHE`).
