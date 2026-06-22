#!/usr/bin/env bash
# Shared Docker install flow — sourced by platform entry scripts after bootstrap.

run_docker_install() {
  source_common_libs yes

  msg_info "Installing Docker"
  setup_docker
  msg_ok "Installed Docker"

  guided_config
  clone_repo
  source_common_libs no
  deploy_stack
}

print_install_done() {
  local ip=${1:-${LOCAL_IP:-127.0.0.1}}
  msg_ok "Installation complete"
  echo -e "${INFO}${YW}Open the site at:${CL}"
  echo -e "${GATEWAY}${BGN}http://${ip}:${HTTP_PORT}${CL}"
  echo -e "${INFO}${YW}Credentials saved to:${CL} ~/site-install.creds"
  echo -e "${INFO}${YW}Edit env and restart:${CL} systemctl restart site"
}
