#!/usr/bin/env bash
# One-time dev setup. Prefer: scripts/dev.sh setup
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/setup.sh" "$@"
