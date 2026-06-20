#!/usr/bin/env bash
# Stop stray local dev processes. Used internally by scripts/dev.sh stop
set -euo pipefail
# shellcheck source=lib/common.sh
source "$(cd "$(dirname "$0")" && pwd)/lib/common.sh"
# shellcheck source=lib/process.sh
source "$(cd "$(dirname "$0")" && pwd)/lib/process.sh"
stop_local_patterns
echo "Local dev processes stopped."
