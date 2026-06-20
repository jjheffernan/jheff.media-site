# jheff.media-site — agent instructions

Automotive photography management — shoots, galleries, client delivery. Rust fullstack: Yew WASM + Actix + MongoDB.

Forked from [yew-fullstack-boilerplate](https://github.com/lukidoescode/yew-fullstack-boilerplate). Frontend follows [Yew 0.23](https://yew.rs/docs/getting-started/introduction) and [yewstack/yew examples](https://github.com/yewstack/yew/tree/master/examples).

## Read first

| Resource | Purpose |
|----------|---------|
| [docs/README.md](docs/README.md) | Human docs index |
| [.agents/README.md](.agents/README.md) | Rules and skills layout |
| [.agents/skills/yew-patterns/SKILL.md](.agents/skills/yew-patterns/SKILL.md) | **Yew + Trunk patterns** |
| [docs/development.md](docs/development.md) | Dev server, env vars |
| [docs/deployment-dev.md](docs/deployment-dev.md) | Deploy and verify dev stack |

## Stack (short)

- **Frontend** — Yew 0.23, **Trunk**, routes: Dashboard, Shoots, Galleries, Profile
- **Backend** — Actix 4, MongoDB 3, JWT (`backend/src/secret.key`, gitignored)
- **Dev** — `scripts/run-dev.sh`; app at http://localhost:8080 (proxies Trunk :8000)

## Build commands

```bash
scripts/setup-dev.sh   # secret.key, wasm target, trunk

cargo build -p jheffmedia-site-backend --features forward-frontend
cd frontend && trunk build
```

## Skills

| Skill | Use |
|-------|-----|
| `dev-server` | Start/stop Docker dev stack |
| `rust-fullstack` | Cross-crate build and debug |
| `yew-patterns` | Yew components, router, Trunk, example references |

Project rules: `.agents/rules/` · Cursor symlinks: `.cursor/skills/`
