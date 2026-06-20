# Frontend rules — Yew + Trunk + WASM

Applies to `frontend/**/*.rs`, `frontend/static/**`, `frontend/index.html`, `frontend/Trunk.toml`.

**Canonical docs:** [docs/frontend.md](../../docs/frontend.md) · [docs/authentication.md](../../docs/authentication.md)

## Lineage and references

Forked from [yew-fullstack-boilerplate](https://github.com/lukidoescode/yew-fullstack-boilerplate) (Actix + Yew + MongoDB + JWT). Frontend patterns follow official [Yew 0.23 docs](https://yew.rs/docs/getting-started/introduction) and the [yewstack/yew examples](https://github.com/yewstack/yew/tree/master/examples) directory — especially `function_router`, `contexts`, `futures`, and `file_upload` for future gallery work.

## Stack

- **Yew 0.23** + **yew-router 0.20** — function components, hooks, `ContextProvider`, `Routable`
- **Trunk** — bundler (`trunk serve`, `trunk build`); see [Trunk getting started](https://trunk-rs.github.io/trunk/)
- **Tailwind CSS v4** — `styles/input.css` → `static/app.css` via `npm run css:build` (Trunk hook)
- HTTP: **gloo-net** + `wasm_bindgen_futures::spawn_local`
- Auth: **AuthContext** in `frontend/src/context/auth.rs`
- Domain: **automotive photography** — shoots, galleries, client delivery

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk   # first time

cd frontend
npm install                   # Tailwind (first time)
trunk serve --open            # dev on :8000
trunk build --release         # output in frontend/dist/
```

Use http://localhost:8080 in dev (Actix proxies Trunk on :8000).

## Yew patterns in this repo

- Context: `context/auth.rs` (`AuthProvider`, `use_context::<AuthContext>`)
- Router: `AppRoutes` + `switch()` — Dashboard, Shoots, Galleries, Profile
- Function components: `Header` uses `#[function_component]`
- Match [yew examples](https://github.com/yewstack/yew/tree/master/examples) when adding new UI (prefer function components + hooks)

## Do not

- Reintroduce webpack as the primary bundler (Trunk stays canonical)
- Add dependencies that pull `stdweb`
- Use `Authentication` header — backend expects `Authorization`
