#!/usr/bin/env bash
# Compatibility shim — Core fetches install/jheffmedia-install.sh.
# Kept so older docs/links still resolve.
BRANCH="${BRANCH:-main}"
REPO_RAW="${REPO_RAW:-https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}}"
# shellcheck disable=SC1090
source <(curl -fsSL "${REPO_RAW}/install/proxmox/install/jheffmedia-install.sh")
