# Project rules — jheff.media-site

## Scope

Minimize diff size. Match existing style in the file you edit. No unrelated refactors, dependency bumps, or “while I’m here” cleanups unless asked.

## Workspace

Cargo workspace members: `frontend`, `backend` (see root `Cargo.toml`, `resolver = "2"`).

| Path | Crate | Notes |
|------|-------|-------|
| `backend/` | `jheffmedia-site-backend` | Dev uses `--features forward-frontend` |
| `frontend/` | `jheffmedia-site-frontend` | WASM target `wasm32-unknown-unknown` |

Legacy env prefix `YEW_FULLSTACK_*` still used in Docker and scripts.

## Secrets

- `backend/src/secret.key` — 32-byte JWT signing key, **gitignored**, required at compile time
- Generate: `openssl rand -out backend/src/secret.key 32`
- Never commit secrets or log tokens

## Scripts (prefer these over ad-hoc commands)

| Script | Use |
|--------|-----|
| `scripts/run-dev.sh` | Full dev stack |
| `scripts/stop-dev.sh` | Tear down dev |
| `scripts/run-dev-force-recreate.sh` | Rebuild after cache/docker issues |
| `scripts/docker-compose.sh` | Wrapper for `docker compose` v2 |

## Docs

- Index: [docs/README.md](../../docs/README.md)
- Architecture: [docs/architecture.md](../../docs/architecture.md)
- API: [docs/api.md](../../docs/api.md)
- Auth: [docs/authentication.md](../../docs/authentication.md)
- Setup: [docs/development.md](../../docs/development.md)

## Commits

Only commit when the user asks. No force-push to `main`.
