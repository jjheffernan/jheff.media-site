#!/usr/bin/env bash
# Stop dev environment. Prefer: scripts/dev.sh stop
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/dev.sh" stop "$@"
