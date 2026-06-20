# Dev server TODO

Track what is needed to run `scripts/run-dev.sh` successfully on a fresh machine.

**Docs:** [README.md](./README.md) · deploy: [deployment-dev.md](./deployment-dev.md) · versions: [version-matrix.md](./version-matrix.md)

## Prerequisites

- [ ] **Rust** — `rustup` with `stable` and `wasm32-unknown-unknown`
- [ ] **Node.js** ≥ 18 and **Yarn**
- [ ] **Docker Desktop** (Compose v2)
- [ ] **JWT secret** — `scripts/setup-dev.sh` or `openssl rand -out backend/src/secret.key 32`

## One-time setup

```bash
scripts/setup-dev.sh
```

- [x] Frontend Yarn deps (included in setup script)
- [ ] Generate `backend/src/secret.key` if missing
- [ ] Confirm Docker: `docker info`
- [ ] Confirm Compose: `scripts/docker-compose.sh version`

## Start dev stack

```bash
scripts/run-dev.sh
```

See [deployment-dev.md](./deployment-dev.md) for verify steps and curl examples.

## Tests

```bash
cargo test -p jheffmedia-site-backend
cargo test -p jheffmedia-site-frontend
```

## If something breaks

[development.md#troubleshooting](./development.md#troubleshooting) · [deployment-dev.md#common-deployment-failures](./deployment-dev.md#common-deployment-failures)

## Fixes applied

- 2026-06-19: MongoDB async, awc, Compose v2 wrapper
- 2026-06-20: Version pins aligned, prod compose env fixed, prod Dockerfile binary name, `mongo:7.0`, unit tests, `scripts/setup-dev.sh`, deployment docs
- 2026-06-20: Yew 0.23 + yew-router 0.20 migration, gloo-net HTTP, webpack 5

## Still open

- [x] Backend unit tests (7)
- [x] Frontend unit tests (2)
- [x] GitHub Actions backend + wasm build with `forward-frontend`
- [ ] Add frontend tests to CI workflow
- [ ] Pin `backend/Dockerfile` Rust image (currently `rust:latest`)
- [ ] Rename legacy docker tags (`yew-fullstack/application`)
- [ ] Yew 0.21 migration
