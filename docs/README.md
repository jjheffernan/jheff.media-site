# Documentation

Human-readable guides for jheff.media-site. Start here or from [README](../README.md).

## Guides

| Doc | What it explains |
|-----|------------------|
| [deployment-dev.md](./deployment-dev.md) | **Deploy dev stack** — setup, Docker, verify, tests |
| [deployment-proxmox.md](./deployment-proxmox.md) | **Proxmox production** — Docker, Immich feed, TLS proxy |
| [version-matrix.md](./version-matrix.md) | Dependency pins, Docker images, env var alignment |
| [architecture.md](./architecture.md) | System layout, dev vs prod, request routing |
| [authentication.md](./authentication.md) | JWT + session flow, login/logout behavior |
| [api.md](./api.md) | REST endpoints, request/response shapes, errors |
| [backend.md](./backend.md) | Actix server, features, env vars, MongoDB |
| [frontend.md](./frontend.md) | Yew app, routing, auth bus, webpack build |
| [development.md](./development.md) | Local setup, troubleshooting |
| [TODO.md](./TODO.md) | Setup checklist and open work |

## For agents

- **[AGENTS.md](../AGENTS.md)** — agent entry point
- **[.agents/](../.agents/)** — rules and skills (symlinked into `.cursor/skills/`)

## Tests

```bash
cargo test -p jheffmedia-site-backend    # 7 unit tests (JWT, serde, API envelope)
cargo test -p jheffmedia-site-frontend   # 2 unit tests (client JSON shapes)
```

## When to update docs

Update the relevant guide when you change:

- API routes or response shapes → `api.md`, `authentication.md`
- Auth or session logic → `authentication.md`, `backend.md`
- Yew routes or client auth → `frontend.md`, `authentication.md`
- Docker, scripts, or env vars → `deployment-dev.md`, `development.md`, `version-matrix.md`
- Dependency pins → `version-matrix.md`, `Cargo.toml` / `package.json`

Keep this index in sync when adding new `docs/*.md` files.
