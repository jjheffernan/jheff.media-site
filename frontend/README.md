# Frontend

Yew 0.23 WASM client (`jheffmedia-site-frontend`), bundled with [Trunk](https://trunk-rs.github.io/trunk/).

**Full reference:** [docs/frontend.md](../docs/frontend.md) · patterns: [.agents/skills/yew-patterns](../.agents/skills/yew-patterns/SKILL.md)

## Quick commands

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk   # first time

cd frontend
trunk serve --open             # dev :8000
trunk build --release          # dist/
```

## Dev tip

Use http://localhost:8080 (backend proxy) so `/api/auth/*` shares origin with the UI.

## Structure

- `index.html`, `Trunk.toml` — Trunk entry
- `src/app.rs`, `router.rs`, `routes/` — dashboard, shoots, galleries, profile
- `src/context/auth.rs` — session + `AuthContext`
- `static/` — CSS

Patterns from [Yew docs](https://yew.rs/docs/getting-started/introduction) and [yewstack examples](https://github.com/yewstack/yew/tree/master/examples).
