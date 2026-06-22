#!/usr/bin/env bash
# Copyright (c) 2026 Jonathan Heffernan
# License: MIT
# Source: https://github.com/jjheffernan/jheff.media-site
#
# Proxmox host — creates an unprivileged LXC via community-scripts build.func.
# One line (Proxmox shell as root):
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/proxmox/ct/jheffmedia.sh)"

source <(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/build.func)

APP="jheffmedia"
var_tags="${var_tags:-media;photography;docker}"
var_cpu="${var_cpu:-2}"
var_ram="${var_ram:-6144}"
var_disk="${var_disk:-24}"
var_os="${var_os:-debian}"
var_version="${var_version:-12}"
var_unprivileged="${var_unprivileged:-1}"
var_brg="${var_brg:-vmbr0}"

BRANCH="${BRANCH:-dev}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
PLATFORM_RAW="${PLATFORM_RAW:-${REPO_RAW}/install/proxmox}"
export BRANCH REPO_RAW

# ponytail: build.func hardcodes ProxmoxVE install URLs; redirect to this repo
__install_url="${PLATFORM_RAW}/container-install.sh"
eval "$(declare -f build_container | sed "s|https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/install/\${var_install}.sh|${__install_url}|g")"

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
  BRANCH="${BRANCH:-dev}"
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
