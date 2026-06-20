#!/usr/bin/env bash
# Run production stack. Prefer: scripts/prod.sh start
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/prod.sh" start "$@"
