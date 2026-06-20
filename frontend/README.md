# Frontend

Yew 0.23 WASM + **Tailwind CSS v4**, bundled with [Trunk](https://trunk-rs.github.io/trunk/).

## Quick commands

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk   # first time

cd frontend
npm install                    # Tailwind CLI
npm run css:build              # or runs via Trunk hook
trunk serve --open             # dev :8000
```

Use http://localhost:8080 in dev (Actix proxies Trunk).

## CSS

- Source: `styles/input.css` (`@import "tailwindcss"`, `@source` scans `src/**/*.rs`)
- Output: `static/app.css` (gitignored — built by Trunk hook / `npm run css:build`)
- UI: Tailwind utility classes in Yew `html!` macros (Rust underneath)

## Component toolbox

Reusable building blocks in `src/components/ui/` (patterns from [yewstack examples](https://github.com/yewstack/yew/tree/master/examples)):

| Component | Use |
|-----------|-----|
| `Button`, `Card`, `Grid`, `Stack` | Layout and actions |
| `Heading`, `Text`, `Section` | Typography |
| `NavLink` | Router-aware nav (`function_router`) |
| `Spinner`, `LazyImage` | Async / `suspense` loading |
| `PhotoFeed` | Homepage grid — fetches `/api/feed`, keyed list |

Photos are streamed from Immich (or static URLs) via the backend proxy — not bundled in this crate.

See [.agents/skills/yew-patterns](../.agents/skills/yew-patterns/SKILL.md) and [docs/frontend.md](../docs/frontend.md).
