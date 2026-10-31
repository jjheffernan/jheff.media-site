#!/usr/bin/env bash
# Compose via Docker or Podman. Override with CONTAINER_RUNTIME.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib/container.sh
source "$SCRIPT_DIR/lib/container.sh"
require_container

if "$CONTAINER_BIN" compose version >/dev/null 2>&1; then
  exec "$CONTAINER_BIN" compose "$@"
fi
if [[ "$CONTAINER_BIN" == podman ]] && command -v podman-compose >/dev/null 2>&1; then
  exec podman-compose "$@"
fi
if command -v docker-compose >/dev/null 2>&1; then
  exec docker-compose "$@"
fi
echo "error: $CONTAINER_BIN has no compose command. Install the compose plugin, or podman-compose." >&2
exit 1
