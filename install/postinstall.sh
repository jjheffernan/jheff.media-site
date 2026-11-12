#!/usr/bin/env bash
# Publish admin bootstrap info after install. Long-lived secrets stay only in
# install/common/.env.production and backend/src/secret.key (chmod 600/400).
#
#   install/postinstall.sh [--fresh] [--show-creds]
#   jheffmedia --postinstall [--show-creds]

set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/opt/jheffmedia-site}"
COMMON_DIR="${INSTALL_DIR}/install/common"
ENV_FILE="${COMMON_DIR}/.env.production"
SECRET_KEY="${INSTALL_DIR}/backend/src/secret.key"
MARKER="${COMMON_DIR}/.postinstall-published"
SERVICE_USER="${SERVICE_USER:-jheffmedia}"

FRESH=0
SHOW_CREDS=0
for arg in "$@"; do
  case "$arg" in
    --fresh) FRESH=1 ;;
    --show-creds) SHOW_CREDS=1 ;;
    --help | -h)
      cat <<'EOF'
Usage: postinstall.sh [--fresh] [--show-creds]

  --fresh       First-run after deploy_stack (prints admin path once)
  --show-creds  Re-print admin bootstrap + secret file paths
                (does not dump Mongo/JWT values — read .env.production)
EOF
      exit 0
      ;;
  esac
done

if [[ "$SHOW_CREDS" -eq 1 && ! -r "$ENV_FILE" ]]; then
  echo "--show-creds needs read access to ${ENV_FILE} (root or ${SERVICE_USER})." >&2
  exit 1
fi

ADMIN_EMAILS=""
HTTP_PORT=8080
HTTP_BIND="0.0.0.0"
if [[ -f "$ENV_FILE" ]]; then
  ADMIN_EMAILS="$(grep -E '^ADMIN_EMAILS=' "$ENV_FILE" | head -1 | cut -d= -f2- || true)"
  HTTP_PORT="$(grep -E '^HTTP_PORT=' "$ENV_FILE" | head -1 | cut -d= -f2- || echo 8080)"
  HTTP_BIND="$(grep -E '^HTTP_BIND=' "$ENV_FILE" | head -1 | cut -d= -f2- || echo 0.0.0.0)"
fi

print_admin_banner() {
  echo "Postinstall — admin access"
  if [[ -n "$ADMIN_EMAILS" ]]; then
    echo "  Admin email(s): ${ADMIN_EMAILS}"
    echo "  Sign up at http://127.0.0.1:${HTTP_PORT}/ with that email (any password ≥8 chars)."
    echo "  First signup matching ADMIN_EMAILS becomes admin — no seed password."
    echo "  Then: Admin → Setup (Immich / Nextcloud / static; no redeploy)."
  else
    echo "  No ADMIN_EMAILS set. Add ADMIN_EMAILS=you@example.com to ${ENV_FILE},"
    echo "  then: systemctl restart site — and sign up with that email."
    echo "  After admin signup: Admin → Setup to choose the photo feed."
  fi
  echo "  Install: ${INSTALL_DIR} (owner ${SERVICE_USER})"
  echo "  Listen:  ${HTTP_BIND}:${HTTP_PORT}"
  echo "  Secrets: ${ENV_FILE} + ${SECRET_KEY} (0600/0400; root and ${SERVICE_USER} can read)"
  echo "  Update later: jheffmedia --update"
}

print_secret_locations() {
  echo
  echo "--- secret locations (values not printed) ---"
  if [[ -f "$ENV_FILE" ]]; then
    echo "  Env (Mongo, ADMIN_EMAILS, Immich keys): ${ENV_FILE}"
    ls -l "$ENV_FILE" 2>/dev/null || true
  else
    echo "  Env file missing: ${ENV_FILE}"
  fi
  if [[ -f "$SECRET_KEY" ]]; then
    echo "  JWT secret key: ${SECRET_KEY}"
    ls -l "$SECRET_KEY" 2>/dev/null || true
  else
    echo "  JWT secret key missing: ${SECRET_KEY}"
  fi
  echo "  To inspect a value: grep '^MONGO_PASSWORD=' ${ENV_FILE}"
  echo "  Threat model: root (and docker-group) on this LXC can always read env."
  echo "  Goal: no /root/*.creds dump, no world-readable secrets, no duplicate stores."
  echo "--- end ---"
}

print_admin_banner

# Drop leftover dump from older installs (duplicate secret store).
for legacy in /root/site-install.creds "${HOME}/site-install.creds" \
  "${INSTALL_DIR}/site-install.creds" "${INSTALL_DIR}/install/site-install.creds"; do
  if [[ -f "$legacy" ]]; then
    rm -f "$legacy"
    echo "  Removed legacy ${legacy} (use ${ENV_FILE} instead)."
  fi
done

should_mark=0
if [[ "$SHOW_CREDS" -eq 1 ]]; then
  print_secret_locations
  should_mark=1
elif [[ "$FRESH" -eq 1 && ! -f "$MARKER" ]]; then
  print_secret_locations
  should_mark=1
fi

if [[ "$should_mark" -eq 1 ]]; then
  mkdir -p "$COMMON_DIR"
  date -Iseconds >"$MARKER"
  chmod 600 "$MARKER" 2>/dev/null || true
elif [[ -f "$MARKER" ]]; then
  echo "  (Admin reminder already shown. Re-show: jheffmedia --postinstall --show-creds)"
fi
