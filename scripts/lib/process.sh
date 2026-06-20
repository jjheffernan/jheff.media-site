#!/usr/bin/env bash
# Background process helpers for local dev.
set -euo pipefail

# shellcheck source=common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

pid_file() {
  echo "$PID_DIR/$1.pid"
}

is_running() {
  local name=$1
  local pf
  pf="$(pid_file "$name")"
  if [[ -f "$pf" ]]; then
    local pid
    pid=$(<"$pf")
    if kill -0 "$pid" 2>/dev/null; then
      return 0
    fi
  fi
  return 1
}

start_background() {
  local name=$1
  shift
  local pf
  pf="$(pid_file "$name")"

  if is_running "$name"; then
    log "Already running: $name (pid $(<"$pf"))"
    return 0
  fi

  log "Starting $name → $LOG_DIR/$name.log"
  nohup "$@" >>"$LOG_DIR/$name.log" 2>&1 &
  echo $! >"$pf"
  log "  pid $(<"$pf")"
}

stop_background() {
  local name=$1
  local pf
  pf="$(pid_file "$name")"
  if [[ -f "$pf" ]]; then
    local pid
    pid=$(<"$pf")
    if kill -0 "$pid" 2>/dev/null; then
      log "Stopping $name (pid $pid)"
      kill "$pid" 2>/dev/null || true
      sleep 1
      kill -9 "$pid" 2>/dev/null || true
    fi
    rm -f "$pf"
  fi
}

wait_for_tcp() {
  local host=$1
  local port=$2
  local timeout=${3:-120}
  local label=${4:-"$host:$port"}
  local elapsed=0

  while (( elapsed < timeout )); do
    if (echo >/dev/tcp/"$host"/"$port") >/dev/null 2>&1; then
      log "Ready: $label (${elapsed}s)"
      return 0
    fi
    log "Waiting: $label … ${elapsed}s / ${timeout}s (see logs if this hangs)"
    sleep 3
    elapsed=$((elapsed + 3))
  done

  log "WARN: $label not ready after ${timeout}s"
  return 1
}

wait_for_http() {
  local url=$1
  local timeout=${2:-180}
  local label=${3:-"$url"}
  local elapsed=0

  while (( elapsed < timeout )); do
    if curl -sf -o /dev/null "$url"; then
      log "Ready: $label (${elapsed}s)"
      return 0
    fi
    log "Waiting: $label … ${elapsed}s / ${timeout}s"
    sleep 3
    elapsed=$((elapsed + 3))
  done

  log "WARN: $label not ready after ${timeout}s — compilation may still be running"
  log "       Run: scripts/dev.sh logs <service>"
  return 1
}

stop_local_patterns() {
  pkill -f 'trunk serve' 2>/dev/null || true
  pkill -f 'tailwindcss.*styles/input.css' 2>/dev/null || true
  pkill -f 'cargo watch.*jheffmedia-site-backend' 2>/dev/null || true
  pkill -f 'target/.*/jheffmedia-site-backend' 2>/dev/null || true
}
