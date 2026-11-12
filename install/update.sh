#!/usr/bin/env bash
# Curl entry → jheffmedia --update
#
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/update.sh)"
#
# Prefer on-host: jheffmedia --update

set -euo pipefail

BRANCH="${BRANCH:-main}"
INSTALL_DIR="${INSTALL_DIR:-/opt/jheffmedia-site}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
CLI_LOCAL="${INSTALL_DIR}/install/common/bin/jheffmedia"

if [[ -x /usr/local/bin/jheffmedia ]]; then
  exec /usr/local/bin/jheffmedia --update "$@"
fi
if [[ -x "$CLI_LOCAL" ]]; then
  exec "$CLI_LOCAL" --update "$@"
fi

# First update on an older install: fetch CLI from raw, then run.
exec bash <(curl -fsSL "${REPO_RAW}/install/common/bin/jheffmedia") --update "$@"
