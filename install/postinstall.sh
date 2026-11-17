#!/usr/bin/env bash
# Publish admin bootstrap info after install. Long-lived secrets stay only in
# install/common/.env.production and backend/src/secret.key (chmod 600/400).
#
#   install/postinstall.sh [--fresh] [--show-creds] [--security]
#   jheffmedia --postinstall [--show-creds] [--security]

set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/opt/jheffmedia-site}"
COMMON_DIR="${INSTALL_DIR}/install/common"
ENV_FILE="${COMMON_DIR}/.env.production"
SECRET_KEY="${INSTALL_DIR}/backend/src/secret.key"
MARKER="${COMMON_DIR}/.postinstall-published"
SERVICE_USER="${SERVICE_USER:-jheffmedia}"

FRESH=0
SHOW_CREDS=0
SECURITY=0
for arg in "$@"; do
  case "$arg" in
    --fresh) FRESH=1 ;;
    --show-creds) SHOW_CREDS=1 ;;
    --security) SECURITY=1 ;;
    --help | -h)
      cat <<'EOF'
Usage: postinstall.sh [--fresh] [--show-creds] [--security]

  --fresh       First-run after deploy_stack (prints admin path once)
  --show-creds  Re-print admin bootstrap + secret file paths
                (does not dump Mongo/JWT values — read .env.production)
  --security    Print a security harden report (no secret values; advisory only)
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
CORS_SET=0
PUBLIC_URL_SET=0
if [[ -f "$ENV_FILE" ]]; then
  ADMIN_EMAILS="$(grep -E '^ADMIN_EMAILS=' "$ENV_FILE" | head -1 | cut -d= -f2- || true)"
  HTTP_PORT="$(grep -E '^HTTP_PORT=' "$ENV_FILE" | head -1 | cut -d= -f2- || echo 8080)"
  HTTP_BIND="$(grep -E '^HTTP_BIND=' "$ENV_FILE" | head -1 | cut -d= -f2- || echo 0.0.0.0)"
  # Presence only — never print values.
  if grep -qE '^YEW_FULLSTACK_CORS_ORIGINS=.+' "$ENV_FILE" 2>/dev/null; then
    CORS_SET=1
  fi
  if grep -qE '^PUBLIC_URL=.+' "$ENV_FILE" 2>/dev/null; then
    PUBLIC_URL_SET=1
  fi
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

# Advisory only — does not rewrite firewall, SSH, or env.
print_security_report() {
  local cors_status public_status
  if [[ "$CORS_SET" -eq 1 ]]; then
    cors_status="set"
  else
    cors_status="unset"
  fi
  if [[ "$PUBLIC_URL_SET" -eq 1 ]]; then
    public_status="set"
  else
    public_status="unset"
  fi

  echo "Postinstall — security harden report (advisory; no auto-changes)"
  echo "  Env file: ${ENV_FILE}"
  if [[ ! -f "$ENV_FILE" ]]; then
    echo "  [!!] Missing .env.production — run install or copy .env.production.example first."
  fi
  echo "  HTTP_BIND=${HTTP_BIND}"
  echo "  HTTP_PORT=${HTTP_PORT}"
  echo "  YEW_FULLSTACK_CORS_ORIGINS: ${cors_status} (value not shown)"
  echo "  PUBLIC_URL: ${public_status} (value not shown)"
  if [[ "$HTTP_BIND" == "0.0.0.0" || "$HTTP_BIND" == "::" || "$HTTP_BIND" == "[::]" ]]; then
    echo "  [!!] HTTP_BIND=${HTTP_BIND} — LAN-exposed on this host."
    echo "       Prefer 127.0.0.1 when a TLS reverse proxy terminates on the same LXC."
  elif [[ "$HTTP_BIND" == "127.0.0.1" || "$HTTP_BIND" == "localhost" ]]; then
    echo "  [ok] HTTP_BIND is loopback — good behind a local reverse proxy."
  else
    echo "  [..] HTTP_BIND=${HTTP_BIND} — confirm this matches your proxy/firewall plan."
  fi
  echo
  echo "--- recommendations (copy/paste; edit as needed) ---"
  echo "  # Bind loopback when Caddy/Nginx/Tunnel is on this LXC"
  echo "  # edit ${ENV_FILE}:"
  echo "  HTTP_BIND=127.0.0.1"
  echo "  # then: systemctl restart site"
  echo
  echo "  # CORS — use the real HTTPS origin (comma-separated for multiple)"
  echo "  YEW_FULLSTACK_CORS_ORIGINS=https://jheff.media"
  echo "  # or set PUBLIC_URL=https://jheff.media before install/update"
  echo
  echo "  # Host firewall (Debian/Ubuntu ufw examples — review before apply)"
  echo "  # ufw default deny incoming"
  echo "  # ufw allow OpenSSH"
  echo "  # ufw allow 80/tcp"
  echo "  # ufw allow 443/tcp"
  echo "  # ufw enable"
  echo
  echo "  # SSH: prefer key-only (PasswordAuthentication no) in /etc/ssh/sshd_config"
  echo "  # Unattended security updates: apt install unattended-upgrades && dpkg-reconfigure -plow unattended-upgrades"
  echo
  echo "  # Threat notes"
  echo "  # - LXC nesting + docker group ≈ root on the host — intentional for day-2 CLI;"
  echo "    treat anyone in docker as fully trusted on this CT."
  echo "  # - GET /metrics is unauthenticated Prometheus text on the app port."
  echo "    Restrict at the reverse proxy (allowlist) or keep HTTP_BIND=127.0.0.1."
  echo "  # - Prefer: jheffmedia --update   over re-running curl|bash install scripts."
  echo "  # - Pin first install: INSTALL_REF=<commit-sha> (or BRANCH=) when curling install.sh"
  echo "--- end security report ---"
  echo "  Docs: docs/deployment-proxmox.md § Security hardening (LXC)"
}

if [[ "$SECURITY" -eq 1 ]]; then
  print_security_report
  # Dedicated mode when only --security was requested.
  if [[ "$FRESH" -eq 0 && "$SHOW_CREDS" -eq 0 ]]; then
    exit 0
  fi
  echo
fi

print_admin_banner

if [[ "$FRESH" -eq 1 ]]; then
  echo "  Harden tips: jheffmedia --postinstall --security"
fi

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
