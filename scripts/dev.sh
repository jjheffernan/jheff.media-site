#!/usr/bin/env bash
# Dev workflow: setup | build | start | stop | logs | status
#
#   scripts/dev.sh setup          One-time machine setup
#   scripts/dev.sh build          Compile check + CSS (local, fast)
#   scripts/dev.sh build --docker Build dev Docker images (verbose, can be slow)
#   scripts/dev.sh start          Local Trunk + backend + Mongo (non-blocking)
#   scripts/dev.sh start --docker Full stack in Docker (detached)
#   scripts/dev.sh stop           Stop local processes + dev containers
#   scripts/dev.sh logs [name]    Tail logs (trunk|backend|tailwind|docker|all)
#   scripts/dev.sh status         Show ports, PIDs, containers
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=lib/process.sh
source "$SCRIPT_DIR/lib/process.sh"

usage() {
  cat <<'EOF'
Usage: scripts/dev.sh <command> [options]

Commands:
  setup              One-time setup (secret, trunk, npm, wasm target)
  build              Local compile check + Tailwind CSS build
  build --docker     Build dev Docker images (verbose)
  build --docker --no-cache
  start              Start local dev (Mongo in Docker, app processes in background)
  start --docker     Start full dev stack in Docker (detached)
  start --build      Run build before start
  start --wait       Wait for :8000/:8080 health checks (default: dispatch only)
  stop               Stop local processes and dev Docker stack
  logs [service]     Tail logs: trunk, backend, tailwind, docker, all
  status             Show what's running

URLs (local):  http://localhost:8080  (app)  http://localhost:8000  (Trunk)

EOF
}

cmd_setup() {
  exec "$SCRIPT_DIR/setup.sh"
}

cmd_build() {
  local mode="local"
  local no_cache=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --docker) mode="docker" ;;
      --no-cache) no_cache="--no-cache" ;;
      *) die "Unknown build option: $1" ;;
    esac
    shift
  done

  ensure_dev_dirs

  if [[ "$mode" == "local" ]]; then
    log_section "Local dev build"
    require_cmd npm
    require_cmd cargo

    log "npm install + css:build"
    (cd "$ROOT/frontend" && npm install && npm run css:build) | tee "$LOG_DIR/build-npm.log"

    log_section "cargo check (backend + frontend)"
    cd "$ROOT"
    cargo check -p jheffmedia-site-backend --features forward-frontend \
      | tee "$LOG_DIR/build-backend.log"
    cargo check -p jheffmedia-site-frontend \
      | tee "$LOG_DIR/build-frontend.log"

    log_section "Local build finished"
    log "Logs: $LOG_DIR/build-*.log"
    return 0
  fi

  log_section "Docker dev image build (verbose — may take 10+ minutes first time)"
  require_cmd docker
  export BUILDKIT_PROGRESS=plain
  export DOCKER_BUILDKIT=1

  cd "$ROOT/scripts"
  "$COMPOSE" -f "$COMPOSE_DEV_FILE" -p "$COMPOSE_DEV_PROJECT" build $no_cache \
    2>&1 | tee "$LOG_DIR/docker-build.log"

  log_section "Docker build finished"
  log "Log: $LOG_DIR/docker-build.log"
  log "Start: scripts/dev.sh start --docker"
}

cmd_start() {
  local mode="local"
  local do_build=false
  local do_wait=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --docker) mode="docker" ;;
      --build) do_build=true ;;
      --wait) do_wait=true ;;
      *) die "Unknown start option: $1" ;;
    esac
    shift
  done

  if [[ "$do_build" == true ]]; then
    if [[ "$mode" == "docker" ]]; then
      cmd_build --docker
    else
      cmd_build
    fi
  fi

  if [[ "$mode" == "docker" ]]; then
    cmd_start_docker "$do_wait"
  else
    cmd_start_local "$do_wait"
  fi
}

cmd_start_local() {
  local do_wait=${1:-false}
  log_section "Starting local dev (non-blocking)"
  ensure_dev_dirs
  source_dev_env

  [[ -f "$ROOT/backend/src/secret.key" ]] || die "Missing secret.key — run: scripts/dev.sh setup"

  require_cmd docker
  require_cmd trunk
  require_cmd npm
  require_cmd cargo

  log_section "MongoDB (Docker)"
  cd "$ROOT/scripts"
  "$COMPOSE" -f "$COMPOSE_DEV_FILE" -p "$COMPOSE_DEV_PROJECT" up -d db \
    2>&1 | tee -a "$LOG_DIR/docker-db.log"

  wait_for_tcp localhost 27017 60 "MongoDB :27017" || true

  unset NO_COLOR

  log_section "Tailwind watch"
  start_background tailwind bash -lc "cd '$ROOT/frontend' && npm run css:watch"

  log_section "Trunk dev server :8000"
  start_background trunk bash -lc "cd '$ROOT/frontend' && unset NO_COLOR && trunk serve --port 8000 --address 0.0.0.0"

  log_section "Actix backend :8080"
  local backend_cmd
  if command -v cargo-watch >/dev/null 2>&1; then
    backend_cmd="cargo watch -x 'run -p jheffmedia-site-backend --features forward-frontend'"
  else
    log "WARN: cargo-watch not installed — using cargo run (install: scripts/dev.sh setup)"
    backend_cmd="cargo run -p jheffmedia-site-backend --features forward-frontend"
  fi
  start_background backend bash -lc "
    cd '$ROOT' &&
    export RUST_LOG='${RUST_LOG:-info}' &&
    unset NO_COLOR &&
    set -a && source '$CONFIG_DEV' && set +a &&
    $backend_cmd
  "

  log_section "Startup dispatched (processes run in background)"
  log "  App:       http://localhost:8080"
  log "  Trunk:     http://localhost:8000"
  log "  Logs:      scripts/dev.sh logs all"
  log "  Status:    scripts/dev.sh status"

  if [[ "$do_wait" == true ]]; then
    wait_for_http "http://localhost:8000/" 90 "Trunk :8000" || true
    wait_for_http "http://localhost:8080/" 120 "App :8080" || true
  else
    log "Skipping health wait (first compile may take a minute). Use: scripts/dev.sh logs trunk"
  fi
}

cmd_start_docker() {
  local do_wait=${1:-false}
  log_section "Starting Docker dev stack (detached)"
  ensure_dev_dirs
  require_cmd docker
  export BUILDKIT_PROGRESS=plain

  cd "$ROOT/scripts"
  log "docker compose up -d --build"
  "$COMPOSE" -f "$COMPOSE_DEV_FILE" -p "$COMPOSE_DEV_PROJECT" up -d --build \
    2>&1 | tee "$LOG_DIR/docker-up.log"

  "$COMPOSE" -f "$COMPOSE_DEV_FILE" -p "$COMPOSE_DEV_PROJECT" ps

  log_section "Docker dev stack started"
  log "  App:   http://localhost:8080"
  log "  Logs:  scripts/dev.sh logs docker"
  if [[ "$do_wait" == true ]]; then
    wait_for_http "http://localhost:8080/" 180 "App :8080" || true
  fi
}

cmd_stop() {
  log_section "Stopping dev environment"
  ensure_dev_dirs

  stop_background tailwind
  stop_background trunk
  stop_background backend
  stop_local_patterns

  if command -v docker >/dev/null 2>&1; then
    cd "$ROOT/scripts"
  "$COMPOSE" -f "$COMPOSE_DEV_FILE" -p "$COMPOSE_DEV_PROJECT" down --remove-orphans \
      2>&1 | tee -a "$LOG_DIR/docker-down.log" || true
  fi

  log "Dev environment stopped"
}

cmd_logs() {
  local service=${1:-all}
  ensure_dev_dirs

  case "$service" in
    trunk)
      tail -f "$LOG_DIR/trunk.log"
      ;;
    backend)
      tail -f "$LOG_DIR/backend.log"
      ;;
    tailwind)
      tail -f "$LOG_DIR/tailwind.log"
      ;;
    docker)
      cd "$ROOT/scripts"
      "$COMPOSE" -f "$COMPOSE_DEV_FILE" -p "$COMPOSE_DEV_PROJECT" logs -f --tail=100
      ;;
    all)
      tail -f "$LOG_DIR/trunk.log" "$LOG_DIR/backend.log" "$LOG_DIR/tailwind.log"
      ;;
    *)
      die "Unknown log target: $service (use trunk|backend|tailwind|docker|all)"
      ;;
  esac
}

cmd_status() {
  ensure_dev_dirs
  log_section "Local processes"
  for name in tailwind trunk backend; do
    if is_running "$name"; then
      log "  $name: running (pid $(<"$(pid_file "$name")"))"
    else
      log "  $name: stopped"
    fi
  done

  log_section "Ports"
  for pair in "8080 App" "8000 Trunk" "27017 MongoDB"; do
    set -- $pair
    port=$1
    label=$2
    if (echo >/dev/tcp/localhost/"$port") >/dev/null 2>&1; then
      log "  :$port $label — open"
    else
      log "  :$port $label — closed"
    fi
  done

  if command -v docker >/dev/null 2>&1; then
    log_section "Docker (yew-fullstack-dev)"
    cd "$ROOT/scripts"
    "$COMPOSE" -f "$COMPOSE_DEV_FILE" -p "$COMPOSE_DEV_PROJECT" ps 2>/dev/null || log "  (no containers)"
  fi

  log_section "Recent log lines (trunk)"
  if [[ -f "$LOG_DIR/trunk.log" ]]; then
    tail -n 5 "$LOG_DIR/trunk.log" >&2 || true
  fi
}

main() {
  local cmd=${1:-help}
  shift || true

  case "$cmd" in
    setup) cmd_setup "$@" ;;
    build) cmd_build "$@" ;;
    start) cmd_start "$@" ;;
    stop) cmd_stop "$@" ;;
    logs) cmd_logs "$@" ;;
    status) cmd_status "$@" ;;
    help|-h|--help) usage ;;
    *) die "Unknown command: $cmd (try: scripts/dev.sh help)" ;;
  esac
}

main "$@"
