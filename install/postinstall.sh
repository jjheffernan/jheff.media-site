#!/usr/bin/env bash
# Publish admin bootstrap info after install. Secrets stay in the local creds
# file (chmod 600); re-runs stay quiet unless --show-creds (root).
#
#   install/postinstall.sh [--fresh] [--show-creds]
#   jheffmedia --postinstall [--show-creds]

set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/opt/jheffmedia-site}"
COMMON_DIR="${INSTALL_DIR}/install/common"
ENV_FILE="${COMMON_DIR}/.env.production"
MARKER="${COMMON_DIR}/.postinstall-published"
CREDS_FILE=""
for candidate in "${HOME}/site-install.creds" /root/site-install.creds; do
  if [[ -f "$candidate" ]]; then
    CREDS_FILE="$candidate"
    break
  fi
done

FRESH=0
SHOW_CREDS=0
for arg in "$@"; do
  case "$arg" in
    --fresh) FRESH=1 ;;
    --show-creds) SHOW_CREDS=1 ;;
    --help | -h)
      cat <<'EOF'
Usage: postinstall.sh [--fresh] [--show-creds]

  --fresh       First-run after deploy_stack (prints admin path + creds once)
  --show-creds  Re-print local site-install.creds (root only)
EOF
      exit 0
      ;;
  esac
done

if [[ "$SHOW_CREDS" -eq 1 && "${EUID:-0}" -ne 0 ]]; then
  echo "--show-creds requires root." >&2
  exit 1
fi

ADMIN_EMAILS=""
if [[ -f "$ENV_FILE" ]]; then
  # shellcheck disable=SC1090
  ADMIN_EMAILS="$(grep -E '^ADMIN_EMAILS=' "$ENV_FILE" | head -1 | cut -d= -f2- || true)"
fi
if [[ -z "$ADMIN_EMAILS" && -n "$CREDS_FILE" ]]; then
  ADMIN_EMAILS="$(grep -E '^Admin emails' "$CREDS_FILE" | sed 's/^Admin emails (ADMIN_EMAILS): //' | head -1 || true)"
fi

HTTP_PORT=8080
if [[ -f "$ENV_FILE" ]]; then
  HTTP_PORT="$(grep -E '^HTTP_PORT=' "$ENV_FILE" | head -1 | cut -d= -f2- || echo 8080)"
fi

echo "Postinstall — admin access"
if [[ -n "$ADMIN_EMAILS" ]]; then
  echo "  Admin email(s): ${ADMIN_EMAILS}"
  echo "  Sign up at http://127.0.0.1:${HTTP_PORT}/ with that email (any password ≥8 chars)."
  echo "  First signup matching ADMIN_EMAILS becomes admin — no seed password."
else
  echo "  No ADMIN_EMAILS set. Add ADMIN_EMAILS=you@example.com to ${ENV_FILE},"
  echo "  then: systemctl restart site — and sign up with that email."
fi
echo "  Creds file: ${CREDS_FILE:-"(missing — re-run install or check ~/site-install.creds)"}"
echo "  Update later: jheffmedia --update"

should_print_creds=0
if [[ "$SHOW_CREDS" -eq 1 ]]; then
  should_print_creds=1
elif [[ "$FRESH" -eq 1 && ! -f "$MARKER" ]]; then
  should_print_creds=1
fi

if [[ "$should_print_creds" -eq 1 ]]; then
  if [[ -n "$CREDS_FILE" && -f "$CREDS_FILE" ]]; then
    echo
    echo "--- ${CREDS_FILE} ---"
    cat "$CREDS_FILE"
    echo "--- end ---"
  else
    echo "No credentials file found."
  fi
  mkdir -p "$COMMON_DIR"
  date -Iseconds >"$MARKER"
  chmod 600 "$MARKER" 2>/dev/null || true
elif [[ -f "$MARKER" ]]; then
  echo "  (Secrets already published. Re-show: jheffmedia --postinstall --show-creds)"
fi
