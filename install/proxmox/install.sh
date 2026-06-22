#!/usr/bin/env bash
# Proxmox installer
#
# New LXC on Proxmox host (root):
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/proxmox/install.sh)"
#
# Existing LXC/VM on Proxmox (root):
#   MODE=existing bash -c "$(curl -fsSL .../install/proxmox/install.sh)"

set -euo pipefail

BRANCH="${BRANCH:-dev}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
PLATFORM_RAW="${PLATFORM_RAW:-${REPO_RAW}/install/proxmox}"

if [[ -f /etc/pve/.version ]] && [[ "${MODE:-auto}" != "existing" ]]; then
  exec bash <(curl -fsSL "${PLATFORM_RAW}/ct/jheffmedia.sh") "$@"
fi

if [[ $EUID -ne 0 ]]; then
  echo "Run as root inside the target LXC/VM (or on the Proxmox host to create a new LXC)." >&2
  exit 1
fi

export FUNCTIONS_FILE_PATH
FUNCTIONS_FILE_PATH="$(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/install.func)"
export BRANCH REPO_RAW
bash -c "$(curl -fsSL "${PLATFORM_RAW}/container-install.sh")"
