---
name: yew-patterns
description: Yew 0.23 + Trunk patterns for jheff.media automotive photography frontend. Use when building Yew components, routing, context, or HTTP from WASM. References official Yew docs and yewstack examples.
---

# Yew patterns skill

## References (read before inventing patterns)

| Resource | Use for |
|----------|---------|
| [Yew getting started](https://yew.rs/docs/getting-started/introduction) | Trunk setup, MSRV 1.84+, wasm target |
| [yewstack/yew examples](https://github.com/yewstack/yew/tree/master/examples) | Router, context, futures, file upload |
| [yew-fullstack-boilerplate](https://github.com/lukidoescode/yew-fullstack-boilerplate) | Original Actix + JWT + Mongo layout |
| [Trunk docs](https://trunk-rs.github.io/trunk/) | `index.html` data-trunk tags, `Trunk.toml` |

## Example map (yewstack/yew)

| Example | Apply when |
|---------|------------|
| `function_router` | New routes or nav changes |
| `contexts` | Shared auth/session state |
| `futures` / `async_clock` | gloo-net HTTP, timers |
| `file_upload` | Future gallery ingest |
| `keyed_list` | Shoot/gallery lists |

Run an example locally: `cd examples/<name> && trunk serve --open`

## This project's layout

```
frontend/
  index.html          # Trunk entry (data-trunk rel="rust")
  Trunk.toml          # serve :8000, dist/
  src/
    app.rs            # BrowserRouter → AuthProvider → Layout → Router
    context/auth.rs   # session + login/logout callbacks
    routes/           # Home (dashboard), Shoots, Galleries, Profile
    components/       # layout, auth forms
  static/             # CSS (theme, components)
```

## Trunk dev

```bash
cd frontend && trunk serve --port 8000 --address 0.0.0.0
```

Production: `trunk build --release` → `frontend/dist/` (served by Actix in prod).

## Automotive photography domain (current shell)

| Route | Purpose |
|-------|---------|
| `/` | Dashboard — shoots, galleries, deliverables overview |
| `/shoots` | Session planning placeholder |
| `/galleries` | Client gallery placeholder |
| `/profile` | Account placeholder |

Backend API unchanged: `/api/auth/*` on same origin via :8080 proxy.

**Docs:** [docs/frontend.md](../../docs/frontend.md) · [.agents/rules/frontend.md](../../.agents/rules/frontend.md)
