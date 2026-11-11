#!/usr/bin/env bash

# Copyright (c) 2026 Jonathan Heffernan
# License: MIT
# Source: https://github.com/jjheffernan/jheff.media-site
#
# In-container install for Proxmox LXC (community-scripts Core lxc/install.func).
# Fetched by build_container as install/jheffmedia-install.sh under this scripts root.

source /dev/stdin <<<"$FUNCTIONS_FILE_PATH"
color
verb_ip6
catch_errors
setting_up_container
network_check
update_os

BRANCH="${BRANCH:-main}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
export BRANCH REPO_RAW
export COMMUNITY_SCRIPTS_CORE_URL="${COMMUNITY_SCRIPTS_CORE_URL:-https://raw.githubusercontent.com/community-scripts/core/main}"

# shellcheck disable=SC1090
source <(curl -fsSL "${REPO_RAW}/install/common/run-install.sh")

run_docker_install

motd_ssh
customize
cleanup_lxc

print_install_done "${LOCAL_IP:-127.0.0.1}"
