# jheff.media-site — agent instructions

Automotive photography management — shoots, galleries, client delivery. Rust fullstack: Yew WASM + Actix + MongoDB.

Forked from [yew-fullstack-boilerplate](https://github.com/lukidoescode/yew-fullstack-boilerplate). Frontend: [Yew 0.23](https://yew.rs/docs/getting-started/introduction) + Trunk.

## Canonical agent context

**Everything lives in [.agents/](.agents/)** — rules, skills, Cursor integration. Do not duplicate long instructions here.

| Resource | Purpose |
|----------|---------|
| [.agents/README.md](.agents/README.md) | Layout, how to add rules/skills |
| [.agents/rules/](.agents/rules/) | Repo conventions (project, backend, frontend) |
| [.agents/skills/](.agents/skills/) | Task workflows and slash skills |
| [.agents/cursor.md](.agents/cursor.md) | Cursor symlinks, `/ponytail`, `/caveman` |
| [docs/README.md](docs/README.md) | Human docs (API, auth, architecture) |

## Quick start

```bash
scripts/dev.sh setup
scripts/dev.sh start    # http://localhost:8080
```

Full dev/troubleshooting: skill [`dev-server`](.agents/skills/dev-server/SKILL.md) · [docs/deployment-dev.md](docs/deployment-dev.md)

## Skills

| Skill | When |
|-------|------|
| [`dev-server`](.agents/skills/dev-server/SKILL.md) | Start/stop/troubleshoot Trunk + Actix + MongoDB |
| [`rust-fullstack`](.agents/skills/rust-fullstack/SKILL.md) | Build, test, debug across frontend + backend crates |
| [`yew-patterns`](.agents/skills/yew-patterns/SKILL.md) | Yew components, router, Trunk, yewstack examples |
| [`ponytail`](.agents/skills/ponytail/SKILL.md) | `/ponytail` — lazy senior dev, YAGNI, minimal diff |
| [`caveman`](.agents/skills/caveman/SKILL.md) | `/caveman` — terse communication, fewer tokens |

## Stack (one line)

Yew 0.23 + Trunk · Actix 4 + MongoDB 3 · JWT in `backend/src/secret.key` (gitignored)

Build/test commands: skill [`rust-fullstack`](.agents/skills/rust-fullstack/SKILL.md).
