#!/usr/bin/env bash
# Bare-metal / generic Linux installer (Debian/Ubuntu, root)
#
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/baremetal/install.sh)"

set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "Run as root on the target host." >&2
  exit 1
fi

BRANCH="${BRANCH:-dev}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"

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
