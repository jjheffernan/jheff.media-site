#!/usr/bin/env bash
# Update an existing install (pull + rebuild Docker stack).
#
# Inside the host/CT where the site is installed (root):
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/update.sh)"
#
# Optional: INSTALL_DIR=/opt/jheffmedia-site BRANCH=main

set -euo pipefail

if [[ ${EUID:-0} -ne 0 ]]; then
  echo "Run as root inside the install host (or LXC)." >&2
  exit 1
fi

INSTALL_DIR="${INSTALL_DIR:-/opt/jheffmedia-site}"
BRANCH="${BRANCH:-main}"
COMPOSE_DIR="${INSTALL_DIR}/install/common"
ENV_FILE="${COMPOSE_DIR}/.env.production"

if [[ ! -d "${INSTALL_DIR}/.git" ]]; then
  echo "No git install at ${INSTALL_DIR}" >&2
  exit 1
fi
if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing ${ENV_FILE} — run the installer first." >&2
  exit 1
fi

echo "[..] Pulling ${BRANCH} in ${INSTALL_DIR}"
git -C "$INSTALL_DIR" fetch origin "$BRANCH"
git -C "$INSTALL_DIR" checkout "$BRANCH"
git -C "$INSTALL_DIR" pull --ff-only origin "$BRANCH"
echo "[OK] Repository updated"

# shellcheck disable=SC1091
source "${INSTALL_DIR}/scripts/lib/container.sh"
require_container

# shellcheck disable=SC1091
set -a
source "$ENV_FILE"
set +a
export HTTP_PORT="${HTTP_PORT:-8080}"

echo "[..] Rebuilding application image (may take several minutes)"
cd "$COMPOSE_DIR"
"$CONTAINER_BIN" compose -f docker-compose.yml --env-file .env.production build
"$CONTAINER_BIN" compose -f docker-compose.yml --env-file .env.production up -d --remove-orphans
systemctl restart site 2>/dev/null || true
echo "${BRANCH}" >"${COMPOSE_DIR}/.install-version"
echo "[OK] Stack updated — http://127.0.0.1:${HTTP_PORT}/"
