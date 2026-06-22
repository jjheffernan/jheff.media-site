#!/usr/bin/env bash
# Shared defaults for install targets (sourced, not executed).

REPO="${REPO:-https://github.com/jjheffernan/jheff.media-site.git}"
BRANCH="${BRANCH:-dev}"
INSTALL_DIR="${INSTALL_DIR:-/opt/jheffmedia-site}"
HTTP_PORT="${HTTP_PORT:-8080}"
DB_NAME="${DB_NAME:-jheffmedia}"
CONTACT_EMAIL="${CONTACT_EMAIL:-hello@jheff.media}"
FEED_PROVIDER="${FEED_PROVIDER:-static}"
PUBLIC_URL="${PUBLIC_URL:-}"

REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
COMMON_DIR="${COMMON_DIR:-${INSTALL_DIR}/install/common}"
COMMON_RAW="${COMMON_RAW:-${REPO_RAW}/install/common}"

# ponytail: demo feed uses external URLs only — no Immich or bundled media required
FEED_STATIC_JSON="${FEED_STATIC_JSON:-[{\"id\":\"1\",\"thumbnailUrl\":\"https://picsum.photos/seed/auto1/800/600\",\"title\":\"Track still\",\"mediaType\":\"image\"},{\"id\":\"2\",\"thumbnailUrl\":\"https://picsum.photos/seed/auto2/800/600\",\"title\":\"Garage detail\",\"mediaType\":\"image\"}]}"

random_alnum() {
  openssl rand -base64 24 | tr -dc 'a-zA-Z0-9' | head -c "${1:-16}"
}

source_common_libs() {
  local from_raw=${1:-no}
  if [[ "$from_raw" == "yes" || ! -f "${COMMON_DIR}/lib/defaults.sh" ]]; then
    # shellcheck disable=SC1090
    source <(curl -fsSL "${COMMON_RAW}/lib/defaults.sh")
    # shellcheck disable=SC1090
    source <(curl -fsSL "${COMMON_RAW}/lib/guided-config.sh")
    # shellcheck disable=SC1090
    source <(curl -fsSL "${COMMON_RAW}/lib/deploy.sh")
  else
    # shellcheck disable=SC1091
    source "${COMMON_DIR}/lib/defaults.sh"
    # shellcheck disable=SC1091
    source "${COMMON_DIR}/lib/guided-config.sh"
    # shellcheck disable=SC1091
    source "${COMMON_DIR}/lib/deploy.sh"
  fi
}
