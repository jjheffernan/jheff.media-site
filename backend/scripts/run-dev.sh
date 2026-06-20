#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export RUST_LOG="${RUST_LOG:-info}"
cargo watch -x "run -p jheffmedia-site-backend --features forward-frontend"
