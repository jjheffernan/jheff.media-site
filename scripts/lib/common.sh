#!/usr/bin/env bash
# Shared paths and logging for scripts/*.sh
set -euo pipefail

_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT="$(cd "$_SCRIPT_DIR/.." && pwd)"

DEV_DIR="$ROOT/.dev"
LOG_DIR="$DEV_DIR/logs"
PID_DIR="$DEV_DIR/pids"

COMPOSE="$ROOT/scripts/docker-compose.sh"
COMPOSE_DEV_FILE="$ROOT/scripts/docker-compose.dev.yml"
COMPOSE_PROD_FILE="$ROOT/scripts/docker-compose.yml"
COMPOSE_DEV_PROJECT="yew-fullstack-dev"
COMPOSE_PROD_PROJECT="yew-fullstack"

CONFIG_DEV="$ROOT/scripts/config/dev.env"

log() {
  echo "[$(date '+%H:%M:%S')] $*" >&2
}

log_section() {
  echo >&2
  echo "== $* ==" >&2
}

die() {
  log "ERROR: $*"
  exit 1
}

ensure_dev_dirs() {
  mkdir -p "$LOG_DIR" "$PID_DIR"
}

require_cmd() {
  local cmd=$1
  command -v "$cmd" >/dev/null 2>&1 || die "Required command not found: $cmd"
}

source_dev_env() {
  if [[ -f "$CONFIG_DEV" ]]; then
    # shellcheck disable=SC1090
    set -a
    source "$CONFIG_DEV"
    set +a
    log "Loaded $CONFIG_DEV"
  else
    log "WARN: $CONFIG_DEV not found — using defaults"
  fi
}
