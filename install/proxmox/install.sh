#!/usr/bin/env bash
# Proxmox installer
#
# New LXC on Proxmox host (root):
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
#
# Existing LXC/VM (Debian template, etc.):
#   MODE=existing bash -c "$(curl -fsSL .../install/proxmox/install.sh)"

set -euo pipefail

BRANCH="${BRANCH:-main}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
PLATFORM_RAW="${PLATFORM_RAW:-${REPO_RAW}/install/proxmox}"
CORE_URL="${COMMUNITY_SCRIPTS_CORE_URL:-https://raw.githubusercontent.com/community-scripts/core/main}"

if [[ -f /etc/pve/.version ]] && [[ "${MODE:-auto}" != "existing" ]]; then
  exec bash <(curl -fsSL "${PLATFORM_RAW}/ct/jheffmedia.sh") "$@"
fi

if [[ $EUID -ne 0 ]]; then
  echo "Run as root inside the target LXC/VM (or on the Proxmox host to create a new LXC)." >&2
  exit 1
fi

# Content of Core install helpers (same shape build_container exports).
FUNCTIONS_FILE_PATH="$(curl -fsSL "${CORE_URL}/lxc/install.func")"
export FUNCTIONS_FILE_PATH
export BRANCH REPO_RAW
export COMMUNITY_SCRIPTS_CORE_URL="$CORE_URL"
bash -c "$(curl -fsSL "${PLATFORM_RAW}/install/jheffmedia-install.sh")"
