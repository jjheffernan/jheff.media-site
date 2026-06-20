#!/usr/bin/env bash
# Build + run production. Prefer: scripts/prod.sh build && scripts/prod.sh start
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
"$DIR/prod.sh" build
exec "$DIR/prod.sh" start "$@"
