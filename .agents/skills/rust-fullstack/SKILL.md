---
name: rust-fullstack
description: Build, test, and debug the jheff.media-site Rust workspace (Yew WASM frontend + Actix backend). Use when compiling crates, fixing dependency/API mismatches, or working across frontend and backend.
---

# Rust fullstack skill

## Workspace

Root `Cargo.toml` — members `frontend`, `backend`.

```bash
# Backend (dev proxy to Trunk)
cargo build -p jheffmedia-site-backend --features forward-frontend
cargo test -p jheffmedia-site-backend

# Frontend WASM
rustup target add wasm32-unknown-unknown
cargo build -p jheffmedia-site-frontend --target wasm32-unknown-unknown

# Frontend bundle (Trunk — recommended)
cargo install --locked trunk
cd frontend && trunk build
```

## Feature flags

| Crate | Feature | When |
|-------|---------|------|
| `jheffmedia-site-backend` | `forward-frontend` | Dev — reverse proxy to Trunk :8000 |
| `jheffmedia-site-backend` | (default) | Serve static `YEW_FULLSTACK_STATIC` |

## Lineage

Based on [yew-fullstack-boilerplate](https://github.com/lukidoescode/yew-fullstack-boilerplate). Frontend follows [Yew 0.23](https://yew.rs/docs/getting-started/introduction) + [yew examples](https://github.com/yewstack/yew/tree/master/examples). See also skill: `yew-patterns`.

## File map

```
backend/src/
  config/app.rs        # /api + default_service forward proxy
  api/account_controller.rs
  models/user.rs
  services/forward_frontend.rs

frontend/
  index.html, Trunk.toml
  src/context/auth.rs
  src/routes/          # dashboard, shoots, galleries, profile
  static/              # CSS
```

## CI reference

`.github/workflows/rust.yml` — `trunk build`, backend with `forward-frontend`, tests both crates.

## When changing APIs

- MongoDB 3: async `.await`, `doc!` filters
- Actix 4: `app_data()`, `cfg.default_service()` for proxy (not `scope("/")` alone)
- Yew 0.23: gloo-net, ContextProvider, Trunk bundler

**Docs:** [docs/README.md](../../docs/README.md) · [docs/backend.md](../../docs/backend.md) · [docs/frontend.md](../../docs/frontend.md)
