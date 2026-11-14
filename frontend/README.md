# Frontend (Yew + Trunk)

WASM UI for jheff.media. Styles via **Tailwind CSS 4** using a **Rust** CLI — no Node/npm.

## Setup

```bash
# from repo root (preferred)
scripts/dev.sh setup

# or manually
cargo install --locked trunk --version 0.21.14
cargo install --locked tailwind --version 0.3.0   # crates.io: pure Rust Tailwind 4.3.x
rustup target add wasm32-unknown-unknown
bash build-css.sh
```

`build-css.sh` prefers `tailwind` (Cargo), then a standalone `tailwindcss` binary on `PATH` (Docker).

## Dev

```bash
scripts/dev.sh start              # Trunk + Actix + Tailwind watch
# or
bash build-css.sh --watch &
trunk serve --port 8000
```

Trunk’s `pre_build` hook runs `build-css.sh` automatically.

## Notes

- Input: `styles/input.css`
- Output: `static/app.css` (gitignored — built by Trunk hook / `build-css.sh`)
- UI kit: `src/components/ui/` (local Yew components; rust-ui is inspiration only)
