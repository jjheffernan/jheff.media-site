#!/usr/bin/env bash
# Docker or Podman. Set CONTAINER_RUNTIME=docker or podman to force one.

container_runtime() {
  if [[ -n "${CONTAINER_RUNTIME:-}" ]]; then
    printf '%s\n' "$CONTAINER_RUNTIME"
    return 0
  fi
  if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    printf '%s\n' docker
    return 0
  fi
  if command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then
    printf '%s\n' podman
    return 0
  fi
  if command -v docker >/dev/null 2>&1; then
    printf '%s\n' docker
    return 0
  fi
  if command -v podman >/dev/null 2>&1; then
    printf '%s\n' podman
    return 0
  fi
  return 1
}

require_container() {
  CONTAINER_BIN="$(container_runtime)" || {
    echo "error: install Docker or Podman (compose included)" >&2
    exit 1
  }
  export CONTAINER_BIN
}
