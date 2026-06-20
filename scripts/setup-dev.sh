#!/usr/bin/env bash
# One-time local dev setup (secret key, Rust target, Trunk).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SECRET="$ROOT/backend/src/secret.key"

if [[ ! -f "$SECRET" ]]; then
  echo "Generating JWT secret at backend/src/secret.key"
  openssl rand -out "$SECRET" 32
else
  echo "JWT secret already exists: backend/src/secret.key"
fi

if command -v rustup >/dev/null 2>&1; then
  rustup target add wasm32-unknown-unknown
  if ! command -v trunk >/dev/null 2>&1; then
    echo "Installing Trunk (Yew bundler)..."
    cargo install --locked trunk
  fi
else
  echo "warn: rustup not found — install Rust from https://rustup.rs"
fi

echo "Setup complete. Start dev stack: scripts/run-dev.sh"
