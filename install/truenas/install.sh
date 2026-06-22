#!/usr/bin/env bash
# TrueNAS SCALE installer (Debian-based app host or jail shell, root)
#
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/truenas/install.sh)"
#
# Alternative: deploy install/common/docker-compose.yml via TrueNAS Custom App
# and mount .env.production from a dataset — see README.md.

set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "Run as root in the TrueNAS shell (or a privileged Docker-capable context)." >&2
  exit 1
fi

BRANCH="${BRANCH:-dev}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
INSTALL_DIR="${INSTALL_DIR:-/mnt/.ix-apps/jheffmedia-site}"
export INSTALL_DIR BRANCH REPO_RAW

export FUNCTIONS_FILE_PATH
FUNCTIONS_FILE_PATH="$(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/install.func)"

source /dev/stdin <<<"$FUNCTIONS_FILE_PATH"
color
catch_errors
network_check
update_os

# shellcheck disable=SC1090
source <(curl -fsSL "${REPO_RAW}/install/common/run-install.sh")

run_docker_install
print_install_done "127.0.0.1"
