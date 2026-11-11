#!/usr/bin/env bash
# Shared Docker install flow — sourced by platform entry scripts after bootstrap.

_bootstrap_common_libs() {
  if declare -f source_common_libs >/dev/null 2>&1; then
    return 0
  fi
  local branch="${BRANCH:-main}"
  local repo_raw="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${branch}}"
  # shellcheck disable=SC1090
  source <(curl -fsSL "${repo_raw}/install/common/lib/defaults.sh")
}

run_docker_install() {
  _bootstrap_common_libs
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
