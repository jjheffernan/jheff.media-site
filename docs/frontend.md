# Frontend

Yew 0.23 WASM client for **automotive photography** workflows: `jheffmedia-site-frontend`.

**Lineage:** [yew-fullstack-boilerplate](https://github.com/lukidoescode/yew-fullstack-boilerplate) · **Patterns:** [Yew docs](https://yew.rs/docs/getting-started/introduction) · [yewstack examples](https://github.com/yewstack/yew/tree/master/examples) · skill: [.agents/skills/yew-patterns](../.agents/skills/yew-patterns/SKILL.md)

## Stack

| Tool | Notes |
|------|-------|
| Yew 0.23 | `Renderer`, hooks, `ContextProvider` |
| yew-router 0.20 | `Routable`, `BrowserRouter`, `Switch` |
| Trunk | Official Yew bundler — `index.html` + `Trunk.toml` |
| gloo-net | HTTP from WASM |
| CSS | `frontend/static/` |

## Routes (automotive photography shell)

| Path | Page |
|------|------|
| `/` | Dashboard — shoots, galleries, deliverables overview |
| `/shoots` | Session planning placeholder |
| `/galleries` | Gallery index placeholder |
| `/profile` | Account placeholder |

## Trunk

Entry: `frontend/index.html` with `<link data-trunk rel="rust" href="Cargo.toml" />`.

```bash
cargo install --locked trunk
cd frontend
trunk serve --port 8000      # dev
trunk build --release        # → dist/
```

Dev: use http://localhost:8080 (Actix proxies Trunk on :8000).

## Auth

`src/context/auth.rs` — `AuthProvider`, session storage, gloo-net to `/api/auth/*`.

See [authentication.md](./authentication.md).

## Legacy

Crate notes in [frontend/README.md](../frontend/README.md).
