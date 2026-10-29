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
| CSS | [design.md](./design.md). Edit `frontend/styles/input.css`. Pages use `card`, `btn`, `field`, `hero`. Tokens (`--accent`, `--radius-card`, `--page-width`) are at the top of that file. |

## Routes (`frontend/src/routes/mod.rs`)

| Path | Page |
|------|------|
| `/` | Home |
| `/content` | Content |
| `/schedule` | Schedule |
| `/social` | Social |
| `/shoots` | Shoots |
| `/shoots/:id` | Shoot detail |
| `/galleries` | Galleries |
| `/galleries/:id` | Gallery detail |
| `/booking` | Booking |
| `/contact` | Contact |
| `/legal/terms` | Terms |
| `/legal/privacy` | Privacy |
| `/other-sites` | Other sites |
| `/account` | Account |
| `/profile` | Profile |
| `/404` | Not found |

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
