#!/usr/bin/env bash

# Copyright (c) 2026 Jonathan Heffernan
# License: MIT
# Source: https://github.com/jjheffernan/jheff.media-site
#
# In-container install for Proxmox LXC (community-scripts Core lxc/install.func).
# Fetched by build_container as install/jheffmedia-install.sh under this scripts root.
# Bust: 2026-11-11-b — keep helpers inline so raw.githubusercontent.com cache cannot
# leave us calling source_common_libs before defaults.sh is loaded.

source /dev/stdin <<<"$FUNCTIONS_FILE_PATH"
color
verb_ip6
catch_errors
setting_up_container
network_check
update_os

# Core exports COMMUNITY_SCRIPTS_URL into the CT (often a commit-SHA raw URL).
# Derive REPO_RAW from that so helper curls match the pinned install script.
BRANCH="${BRANCH:-main}"
if [[ -z "${REPO_RAW:-}" && -n "${COMMUNITY_SCRIPTS_URL:-}" ]]; then
  REPO_RAW="${COMMUNITY_SCRIPTS_URL%/install/proxmox}"
fi
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
export BRANCH REPO_RAW
export COMMUNITY_SCRIPTS_CORE_URL="${COMMUNITY_SCRIPTS_CORE_URL:-https://raw.githubusercontent.com/community-scripts/core/main}"

# Cache-bust raw.githubusercontent.com (max-age=300).
_RAW_BUST="${RAW_BUST:-$(date +%s)}"

_load_raw() {
  local rel="$1"
  local url="${REPO_RAW}/${rel}?${_RAW_BUST}"
  local content
  content="$(curl -fsSL -H 'Cache-Control: no-cache' "$url")" || {
    msg_error "Failed to download ${url}"
    exit 1
  }
  [[ -n "$content" ]] || {
    msg_error "Empty download ${url}"
    exit 1
  }
  # shellcheck disable=SC1090
  source /dev/stdin <<<"$content"
}

_load_raw "install/common/lib/defaults.sh"
_load_raw "install/common/lib/guided-config.sh"
_load_raw "install/common/lib/deploy.sh"

if ! declare -f source_common_libs >/dev/null 2>&1; then
  msg_error "defaults.sh loaded but source_common_libs is missing"
  exit 1
fi
if ! declare -f guided_config >/dev/null 2>&1; then
  msg_error "guided-config.sh loaded but guided_config is missing"
  exit 1
fi
if ! declare -f deploy_stack >/dev/null 2>&1; then
  msg_error "deploy.sh loaded but deploy_stack is missing"
  exit 1
fi

msg_info "Installing Docker"
setup_docker
msg_ok "Installed Docker"

guided_config
clone_repo

# Repo is on disk now — prefer local libs for the rest.
COMMON_DIR="${INSTALL_DIR}/install/common"
# shellcheck disable=SC1091
source "${COMMON_DIR}/lib/defaults.sh"
# shellcheck disable=SC1091
source "${COMMON_DIR}/lib/guided-config.sh"
# shellcheck disable=SC1091
source "${COMMON_DIR}/lib/deploy.sh"

deploy_stack

motd_ssh
customize
cleanup_lxc

msg_ok "Installation complete"
echo -e "${INFO}${YW}Open the site at:${CL}"
echo -e "${GATEWAY}${BGN}http://${LOCAL_IP:-127.0.0.1}:${HTTP_PORT}${CL}"
echo -e "${INFO}${YW}Credentials saved to:${CL} ~/site-install.creds"
echo -e "${INFO}${YW}Edit env and restart:${CL} systemctl restart site"
echo -e "${INFO}${YW}Update later:${CL} jheffmedia --update"
echo -e "${INFO}${YW}Admin:${CL} sign up with ADMIN_EMAILS → Admin → Setup"
echo -e "${INFO}${YW}Admin reminder:${CL} jheffmedia --postinstall"
echo -e "${INFO}${YW}Or:${CL} bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/update.sh)\""
