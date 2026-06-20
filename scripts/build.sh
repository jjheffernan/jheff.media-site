#!/usr/bin/env bash
# Production Docker image build. Prefer: scripts/prod.sh build
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/prod.sh" build "$@"
