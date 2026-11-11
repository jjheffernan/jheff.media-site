#!/usr/bin/env bash
# Copyright (c) 2026 Jonathan Heffernan
# License: MIT
# Source: https://github.com/jjheffernan/jheff.media-site
#
# Proxmox host — creates an unprivileged LXC via community-scripts Core.
# One line (Proxmox shell as root):
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/ct/jheffmedia.sh)"

BRANCH="${BRANCH:-main}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
# Scripts root for this app: install/proxmox/{ct,install}/ — matches Core's layout.
export COMMUNITY_SCRIPTS_URL="${COMMUNITY_SCRIPTS_URL:-${REPO_RAW}/install/proxmox}"
export COMMUNITY_SCRIPTS_CORE_URL="${COMMUNITY_SCRIPTS_CORE_URL:-https://raw.githubusercontent.com/community-scripts/core/main}"
export BRANCH REPO_RAW

_cs_boot="${COMMUNITY_SCRIPTS_CORE_DIR:-}/core/build.func"
# shellcheck disable=SC1090
source "$_cs_boot" 2>/dev/null || source <(curl -fsSL "${COMMUNITY_SCRIPTS_CORE_URL}/core/build.func")

APP="jheffmedia"
var_tags="${var_tags:-media;photography;docker}"
var_cpu="${var_cpu:-2}"
var_ram="${var_ram:-6144}"
var_disk="${var_disk:-24}"
var_os="${var_os:-debian}"
var_version="${var_version:-13}"
var_unprivileged="${var_unprivileged:-1}"
var_brg="${var_brg:-vmbr0}"

header_info "$APP"
variables
color
catch_errors

function update_script() {
  header_info
  check_container_storage
  check_container_resources

  if [[ ! -d /opt/jheffmedia-site ]]; then
    msg_error "No ${APP} installation found under /opt/jheffmedia-site"
    exit
  fi

  INSTALL_DIR=/opt/jheffmedia-site
  BRANCH="${BRANCH:-main}"
  COMMON_DIR="${INSTALL_DIR}/install/common"
  # shellcheck disable=SC1091
  source "${COMMON_DIR}/lib/defaults.sh"
  # shellcheck disable=SC1091
  source "${COMMON_DIR}/lib/deploy.sh"
  if [[ -f "${COMMON_DIR}/.env.production" ]]; then
    set -a
    # shellcheck disable=SC1091
    source "${COMMON_DIR}/.env.production"
    set +a
  fi

  update_stack
  msg_ok "Updated ${APP}"
  exit
}

start
build_container
description

msg_ok "Completed successfully!\n"
echo -e "${CREATING}${GN}${APP} setup has been successfully initialized!${CL}"
echo -e "${INFO}${YW}Access the site at:${CL}"
echo -e "${GATEWAY}${BGN}http://${IP}:8080${CL}"
echo -e "${INFO}${YW}Credentials on the container:${CL} /root/site-install.creds"
echo -e "${INFO}${YW}Re-run guided env edits inside CT, then:${CL} systemctl restart site"
