#!/usr/bin/env bash
# Post-deploy smoke checks: homepage + auth endpoint reachability.
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8080}"
TIMEOUT="${TIMEOUT:-10}"

die() {
  echo "smoke: ERROR: $*" >&2
  exit 1
}

check_http() {
  local label=$1
  local url=$2
  local expect_code=$3
  local code
  code=$(curl -sS -o /dev/null -w "%{http_code}" --max-time "$TIMEOUT" "$url") || \
    die "$label: request failed ($url)"
  if [[ "$code" != "$expect_code" ]]; then
    die "$label: expected HTTP $expect_code, got $code ($url)"
  fi
  echo "smoke: OK $label ($code)"
}

check_login_health() {
  local body
  body=$(curl -sS --max-time "$TIMEOUT" \
    -X POST "${BASE_URL}/api/auth/login" \
    -H 'Content-Type: application/json' \
    -d '{"emailOrUsername":"__smoke__","password":"__smoke__"}') || \
    die "login health: request failed"

  if ! echo "$body" | grep -q '"message"'; then
    die "login health: unexpected response (missing message field)"
  fi
  echo "smoke: OK login health (auth API responding)"
}

echo "smoke: checking ${BASE_URL}"

check_http "homepage" "${BASE_URL}/" "200"
check_login_health

echo "smoke: all checks passed"
