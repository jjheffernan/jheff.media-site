#!/usr/bin/env bash
# Stop production stack. Prefer: scripts/prod.sh stop
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/prod.sh" stop "$@"
