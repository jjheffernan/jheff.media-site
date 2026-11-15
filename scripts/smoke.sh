#!/usr/bin/env bash
# Post-deploy smoke checks: homepage, /healthz, auth, optional Immich feed/thumbs.
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

# True when operator env asks for Immich (Mongo site_config can also enable it at runtime).
immich_env_configured() {
  local provider="${YEW_FULLSTACK_FEED_PROVIDER:-${FEED_PROVIDER:-}}"
  local url="${YEW_FULLSTACK_IMMICH_URL:-${IMMICH_URL:-}}"
  local key="${YEW_FULLSTACK_IMMICH_API_KEY:-${IMMICH_API_KEY:-}}"
  [[ "$provider" == "immich" && -n "$url" && -n "$key" ]]
}

# Parse /api/media/feed JSON → "count|thumb_path|sources" (thumb may be empty).
# Prefers Immich items; thumbnail paths are /api/media/thumbnail/….
parse_immich_feed() {
  python3 -c '
import json, sys
raw = sys.stdin.read()
try:
    data = json.loads(raw)
except json.JSONDecodeError:
    print("0||")
    sys.exit(0)
if not isinstance(data, dict) or "items" not in data:
    # Auth envelope / UNKNOWN_SOURCE — treat as no Immich items
    print("0||")
    sys.exit(0)
items = data.get("items") or []
immich = [i for i in items if isinstance(i, dict) and i.get("source") == "immich"]
pool = immich if immich else []
thumb = ""
for item in pool:
    u = item.get("thumbnailUrl") or ""
    if "/api/media/thumbnail/" in u:
        thumb = u
        break
sources = ",".join(data.get("sources") or [])
print(f"{len(immich)}|{thumb}|{sources}")
'
}

abs_url() {
  local path=$1
  if [[ "$path" == http://* || "$path" == https://* ]]; then
    printf '%s\n' "$path"
  else
    printf '%s%s\n' "${BASE_URL}" "$path"
  fi
}

# Skip when Immich is not configured. Fail when env demands Immich but feed/thumbs break.
check_immich_feed() {
  local body parsed count thumb sources code ctype tmp
  body=$(curl -sS --max-time "$TIMEOUT" \
    "${BASE_URL}/api/media/feed?limit=8&source=immich") || \
    die "immich feed: request failed"

  parsed=$(printf '%s' "$body" | parse_immich_feed)
  count=${parsed%%|*}
  rest=${parsed#*|}
  thumb=${rest%%|*}
  sources=${rest#*|}

  if [[ "$count" -eq 0 || -z "$thumb" ]]; then
    if immich_env_configured; then
      die "immich feed: YEW_FULLSTACK_FEED_PROVIDER=immich (+ URL/key) set, but /api/media/feed?source=immich returned no Immich items/thumbs (sources=${sources:-none}; check album id, reachability, rebuilt backend)"
    fi
    echo "smoke: SKIP Immich feed (not configured — set YEW_FULLSTACK_FEED_PROVIDER=immich + YEW_FULLSTACK_IMMICH_URL/API_KEY[/ALBUM_ID] against a live Immich, then re-run)"
    return 0
  fi

  tmp=$(mktemp)
  code=$(curl -sS -o "$tmp" -w "%{http_code}" --max-time "$TIMEOUT" "$(abs_url "$thumb")") || \
    { rm -f "$tmp"; die "immich thumbnail: request failed ($thumb)"; }
  ctype=$(file -b --mime-type "$tmp" 2>/dev/null || true)

  if [[ "$code" != "200" ]]; then
    rm -f "$tmp"
    die "immich thumbnail: expected HTTP 200, got $code ($thumb)"
  fi
  # Prefer MIME; fall back to magic (some proxies send octet-stream).
  if [[ "$ctype" != image/* ]] && ! file "$tmp" 2>/dev/null | grep -qiE 'JPEG|PNG|WebP|image'; then

    rm -f "$tmp"
    die "immich thumbnail: expected image bytes, got ${ctype:-unknown} ($thumb)"
  fi
  rm -f "$tmp"
  echo "smoke: OK Immich feed ($count item(s), thumbnail $thumb)"
}

echo "smoke: checking ${BASE_URL}"

check_http "homepage" "${BASE_URL}/" "200"
check_http "healthz" "${BASE_URL}/healthz" "200"
check_login_health
check_http "media feed" "${BASE_URL}/api/media/feed?limit=1" "200"
check_immich_feed

echo "smoke: all checks passed"
