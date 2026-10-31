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
  build   Build the production image (Docker or Podman)
  start   Run the production stack (compose)
  stop    Tear down production stack

EOF
}

cmd_build() {
  require_container
  log_section "Production image build ($CONTAINER_BIN)"
  export BUILDKIT_PROGRESS=plain
  export DOCKER_BUILDKIT=1

  ensure_dev_dirs
  rm -rf "$ROOT/frontend/dist"

  "$CONTAINER_BIN" build -t jheffmedia/application -t jheffmedia/application:latest "$ROOT" \
    2>&1 | tee "$LOG_DIR/prod-docker-build.log"

  log "Image built: jheffmedia/application"
  log "Log: $LOG_DIR/prod-docker-build.log"
}

require_prod_env() {
  local env_file="$ROOT/scripts/.env.production"
  [[ -f "$env_file" ]] || die "Missing $env_file — copy install/common/.env.production.example and set MONGO_PASSWORD"
  echo "$env_file"
}

cmd_start() {
  log_section "Starting production stack"
  require_container
  local env_file
  env_file="$(require_prod_env)"
  cd "$ROOT/scripts"
  "$COMPOSE" --env-file "$env_file" -f "$COMPOSE_PROD_FILE" -p "$COMPOSE_PROD_PROJECT" up \
    --force-recreate --remove-orphans \
    --exit-code-from application \
    --ignore-pull-failures
}

cmd_stop() {
  log_section "Stopping production stack"
  require_container
  local env_file
  env_file="$(require_prod_env)"
  "$COMPOSE" --env-file "$env_file" -f "$COMPOSE_PROD_FILE" down -v --rmi all --remove-orphans
  cd "$ROOT/scripts"
  "$COMPOSE" --env-file "$env_file" -f "$COMPOSE_PROD_FILE" -p "$COMPOSE_PROD_PROJECT" kill 2>/dev/null || true
  "$COMPOSE" --env-file "$env_file" -f "$COMPOSE_PROD_FILE" -p "$COMPOSE_PROD_PROJECT" rm -f 2>/dev/null || true
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
