#!/usr/bin/env bash
# One-time machine setup: JWT secret, Rust wasm target, Trunk, Tailwind, cargo-watch.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

log_section "Dev setup"

SECRET="$ROOT/backend/src/secret.key"
if [[ ! -f "$SECRET" ]]; then
  log "Generating JWT secret at backend/src/secret.key"
  openssl rand -out "$SECRET" 32
else
  log "JWT secret already exists: backend/src/secret.key"
fi

if command -v rustup >/dev/null 2>&1; then
  log "Adding wasm32-unknown-unknown target"
  rustup target add wasm32-unknown-unknown
  if ! command -v trunk >/dev/null 2>&1; then
    log_section "Installing Trunk (this can take several minutes)"
    cargo install --locked trunk
  else
    log "Trunk already installed: $(command -v trunk)"
  fi
  if ! command -v cargo-watch >/dev/null 2>&1; then
    log_section "Installing cargo-watch"
    cargo install cargo-watch
  else
    log "cargo-watch already installed"
  fi
else
  log "WARN: rustup not found — install Rust from https://rustup.rs"
fi

if command -v npm >/dev/null 2>&1; then
  log_section "Frontend npm dependencies"
  (cd "$ROOT/frontend" && npm install && npm run css:build)
else
  log "WARN: npm not found — install Node.js for Tailwind CSS"
fi

ensure_dev_dirs
log_section "Setup complete"
echo "Setup complete. Start dev: scripts/dev.sh build && scripts/dev.sh start"
