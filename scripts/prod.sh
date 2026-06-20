#!/usr/bin/env bash
# Production workflow: build | start | stop
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

usage() {
  cat <<'EOF'
Usage: scripts/prod.sh <command>

Commands:
  build   Build production Docker image (verbose)
  start   Run production stack (docker compose)
  stop    Tear down production stack

EOF
}

cmd_build() {
  log_section "Production Docker build"
  require_cmd docker
  export BUILDKIT_PROGRESS=plain
  export DOCKER_BUILDKIT=1

  ensure_dev_dirs
  rm -rf "$ROOT/frontend/dist"

  docker build -t yew-fullstack/application -t yew-fullstack/application:latest "$ROOT" \
    2>&1 | tee "$LOG_DIR/prod-docker-build.log"

  log "Image built: yew-fullstack/application"
  log "Log: $LOG_DIR/prod-docker-build.log"
}

cmd_start() {
  log_section "Starting production stack"
  require_cmd docker
  cd "$ROOT/scripts"
  "$COMPOSE" -f "$COMPOSE_PROD_FILE" -p "$COMPOSE_PROD_PROJECT" up \
    --force-recreate --remove-orphans \
    --exit-code-from application \
    --ignore-pull-failures
}

cmd_stop() {
  log_section "Stopping production stack"
  require_cmd docker
  "$COMPOSE" -f "$COMPOSE_PROD_FILE" down -v --rmi all --remove-orphans
  cd "$ROOT/scripts"
  "$COMPOSE" -f "$COMPOSE_PROD_FILE" -p "$COMPOSE_PROD_PROJECT" kill 2>/dev/null || true
  "$COMPOSE" -f "$COMPOSE_PROD_FILE" -p "$COMPOSE_PROD_PROJECT" rm -f 2>/dev/null || true
  log "Production stack stopped"
}

main() {
  local cmd=${1:-help}
  shift || true

  case "$cmd" in
    build) cmd_build "$@" ;;
    start) cmd_start "$@" ;;
    stop) cmd_stop "$@" ;;
    help|-h|--help) usage ;;
    *) die "Unknown command: $cmd" ;;
  esac
}

main "$@"
