#!/usr/bin/env bash
# Rebuild dev Docker images and start. Prefer: scripts/dev.sh build --docker --no-cache && scripts/dev.sh start --docker
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
"$DIR/dev.sh" build --docker --no-cache
exec "$DIR/dev.sh" start --docker "$@"
