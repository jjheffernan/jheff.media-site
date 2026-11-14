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
chmod 600 "$SECRET" 2>/dev/null || true

if command -v rustup >/dev/null 2>&1; then
  log "Adding wasm32-unknown-unknown target"
  rustup target add wasm32-unknown-unknown
  if ! command -v trunk >/dev/null 2>&1; then
    log_section "Installing Trunk (this can take several minutes)"
    cargo install --locked trunk --version 0.21.14
  else
    log "Trunk already installed: $(command -v trunk)"
  fi
  if ! command -v cargo-watch >/dev/null 2>&1; then
    log_section "Installing cargo-watch"
    cargo install cargo-watch
  else
    log "cargo-watch already installed"
  fi
  if ! command -v tailwind >/dev/null 2>&1; then
    log_section "Installing Tailwind CSS CLI (crates.io: tailwind)"
    # Pure Rust Tailwind 4.3.x compiler — no Node/npm.
    cargo install --locked tailwind --version 0.3.0
  else
    log "tailwind already installed: $(command -v tailwind)"
  fi
else
  log "WARN: rustup not found — install Rust from https://rustup.rs"
fi

log_section "Frontend CSS"
(cd "$ROOT/frontend" && bash build-css.sh)

ensure_dev_dirs
log_section "Setup complete"
echo "Setup complete. Start dev: scripts/dev.sh build && scripts/dev.sh start"
