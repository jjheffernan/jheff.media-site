#!/usr/bin/env bash
# Build static/app.css via Tailwind CLI (standalone binary or npm local install).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

if [[ -x "$ROOT/node_modules/.bin/tailwindcss" ]]; then
  BIN="$ROOT/node_modules/.bin/tailwindcss"
elif command -v tailwindcss >/dev/null 2>&1; then
  BIN="$(command -v tailwindcss)"
else
  echo "tailwindcss not found — npm install in frontend/, or put the standalone binary on PATH" >&2
  exit 1
fi

case "${1:---minify}" in
  --watch) exec "$BIN" -i ./styles/input.css -o ./static/app.css --watch ;;
  *) exec "$BIN" -i ./styles/input.css -o ./static/app.css --minify ;;
esac
