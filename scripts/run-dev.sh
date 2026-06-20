#!/usr/bin/env bash
# Start dev stack in Docker (detached). Prefer: scripts/dev.sh start --docker
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/dev.sh" start --docker "$@"
