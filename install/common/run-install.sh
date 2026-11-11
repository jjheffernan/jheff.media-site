#!/usr/bin/env bash
# Shared Docker install flow — sourced by baremetal/TrueNAS entry scripts.
# Proxmox in-LXC install is self-contained in install/proxmox/install/jheffmedia-install.sh.

_load_raw() {
  local rel="$1"
  local branch="${BRANCH:-main}"
  local repo_raw="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${branch}}"
  local bust="${RAW_BUST:-$(date +%s)}"
  local url="${repo_raw}/${rel}?${bust}"
  local content
  content="$(curl -fsSL -H 'Cache-Control: no-cache' "$url")" || {
    echo "Failed to download ${url}" >&2
    return 1
  }
  [[ -n "$content" ]] || {
    echo "Empty download ${url}" >&2
    return 1
  }
  # shellcheck disable=SC1090
  source /dev/stdin <<<"$content"
}

_bootstrap_common_libs() {
  if declare -f source_common_libs >/dev/null 2>&1 \
    && declare -f guided_config >/dev/null 2>&1 \
    && declare -f deploy_stack >/dev/null 2>&1; then
    return 0
  fi
  _load_raw "install/common/lib/defaults.sh"
  _load_raw "install/common/lib/guided-config.sh"
  _load_raw "install/common/lib/deploy.sh"
  declare -f source_common_libs >/dev/null 2>&1 || {
    echo "defaults.sh did not define source_common_libs" >&2
    return 1
  }
}

run_docker_install() {
  _bootstrap_common_libs

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
