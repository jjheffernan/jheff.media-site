#!/usr/bin/env bash
# Guided whiptail/read prompts (all install targets).

prompt_input() {
  local title=$1 prompt=$2 default=${3:-}
  if command -v whiptail >/dev/null 2>&1; then
    whiptail --backtitle "Site install" --title "$title" --inputbox "$prompt" 12 72 "$default" 3>&1 1>&2 2>&3
  else
    local value
    read -rp "$prompt [$default]: " value
    printf '%s' "${value:-$default}"
  fi
}

prompt_password() {
  local title=$1 prompt=$2 default=${3:-}
  if command -v whiptail >/dev/null 2>&1; then
    whiptail --backtitle "Site install" --title "$title" --passwordbox "$prompt" 12 72 "$default" 3>&1 1>&2 2>&3
  else
    local value
    read -rsp "$prompt: " value
    echo >&2
    printf '%s' "${value:-$default}"
  fi
}

prompt_menu() {
  local title=$1 prompt=$2 default=$3
  shift 3
  if command -v whiptail >/dev/null 2>&1; then
    whiptail --backtitle "Site install" --title "$title" --menu "$prompt" 16 72 8 "$@" 3>&1 1>&2 2>&3
  else
    local choice=$default
    echo "$prompt" >&2
    while [[ $# -ge 2 ]]; do
      echo "  $1) $2" >&2
      shift 2
    done
    read -rp "Choice [$default]: " choice
    printf '%s' "${choice:-$default}"
  fi
}

guided_config() {
  if [[ "${UNATTENDED:-}" == "yes" ]]; then
    msg_info "Using unattended defaults (UNATTENDED=yes)"
    [[ -z "${MONGO_PASSWORD:-}" ]] && MONGO_PASSWORD="$(random_alnum 20)"
    msg_ok "Defaults loaded"
    return 0
  fi

  msg_info "Guided setup — repository"
  REPO="$(prompt_input "Repository" "Git clone URL:" "$REPO")"
  BRANCH="$(prompt_input "Branch" "Git branch to deploy:" "$BRANCH")"
  INSTALL_DIR="$(prompt_input "Install path" "Directory on this host:" "$INSTALL_DIR")"
  COMMON_DIR="${INSTALL_DIR}/install/common"
  REPO_RAW="https://raw.githubusercontent.com/jjheffernan/jheff.media-site/${BRANCH}"
  COMMON_RAW="${REPO_RAW}/install/common"
  msg_ok "Repository settings saved"

  msg_info "Guided setup — site"
  HTTP_PORT="$(prompt_input "HTTP port" "Port published on this host (reverse-proxy target):" "$HTTP_PORT")"
  PUBLIC_URL="$(prompt_input "Public URL" "Optional https URL for docs (Enter to skip):" "$PUBLIC_URL")"
  CONTACT_EMAIL="$(prompt_input "Contact email" "Contact email shown on site:" "$CONTACT_EMAIL")"
  msg_ok "Site settings saved"

  msg_info "Guided setup — database"
  msg_custom "ℹ️" "$YW" "MongoDB runs in Docker on this host. No external DB required."
  local mongo_default
  mongo_default="$(random_alnum 20)"
  MONGO_PASSWORD="$(prompt_password "MongoDB" "MongoDB root password (blank = generate):" "$mongo_default")"
  [[ -z "$MONGO_PASSWORD" ]] && MONGO_PASSWORD="$mongo_default"
  DB_NAME="$(prompt_input "Database name" "MongoDB database name:" "$DB_NAME")"
  msg_ok "Database settings saved"

  msg_info "Guided setup — photo feed"
  msg_custom "ℹ️" "$YW" "Choose static/none to run without Immich. Add Immich later in .env.production."
  FEED_PROVIDER="$(prompt_menu "Photo feed" "Homepage feed provider:" "$FEED_PROVIDER" \
    "none" "No homepage feed" \
    "static" "Demo stills (external URLs)" \
    "immich" "Immich album (remote API)")"
  case "$FEED_PROVIDER" in
    immich)
      IMMICH_URL="$(prompt_input "Immich URL" "Immich base URL (https://photos.example.com):" "${IMMICH_URL:-}")"
      IMMICH_API_KEY="$(prompt_password "Immich API key" "Immich API key:" "${IMMICH_API_KEY:-}")"
      IMMICH_ALBUM_ID="$(prompt_input "Immich album" "Album UUID for homepage feed:" "${IMMICH_ALBUM_ID:-}")"
      ;;
    static)
      msg_custom "ℹ️" "$YW" "Using bundled demo JSON. Override FEED_STATIC_JSON in .env.production later."
      ;;
  esac
  msg_ok "Photo feed settings saved"

  msg_info "Guided setup — admin (optional)"
  ADMIN_EMAILS="$(prompt_input "Admin emails" "Comma-separated signup emails granted admin (Enter to skip):" "${ADMIN_EMAILS:-}")"
  msg_ok "Admin settings saved"
}
