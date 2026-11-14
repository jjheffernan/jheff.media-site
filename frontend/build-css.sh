#!/usr/bin/env bash
# Build static/app.css via Tailwind — Rust toolchain preferred (no Node/npm).
#
# Prefer:
#   1. `tailwind` from Cargo (`cargo install tailwind --locked`)
#   2. Official standalone `tailwindcss` binary (Docker / PATH)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

resolve_bin() {
  if [[ -n "${TAILWIND_BIN:-}" && -x "${TAILWIND_BIN}" ]]; then
    echo "${TAILWIND_BIN}"
    return 0
  fi
  if [[ -x "$ROOT/bin/tailwind" ]]; then
    echo "$ROOT/bin/tailwind"
    return 0
  fi
  if command -v tailwind >/dev/null 2>&1; then
    command -v tailwind
    return 0
  fi
  if command -v tailwindcss >/dev/null 2>&1; then
    command -v tailwindcss
    return 0
  fi
  return 1
}

if ! BIN="$(resolve_bin)"; then
  echo "tailwind CLI not found." >&2
  echo "Install with Cargo (preferred):" >&2
  echo "  cargo install tailwind --locked --version 0.3.0" >&2
  echo "Or put the official standalone tailwindcss binary on PATH." >&2
  exit 1
fi

case "${1:---minify}" in
  --watch) exec "$BIN" -i ./styles/input.css -o ./static/app.css --watch ;;
  *) exec "$BIN" -i ./styles/input.css -o ./static/app.css --minify ;;
esac
