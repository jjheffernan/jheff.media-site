#!/usr/bin/env bash
# Write env, compose, systemd, and start the Docker stack.

# Day-2 CLI may source this without community-scripts helpers.
if ! declare -F msg_ok >/dev/null 2>&1; then
  msg_ok() { echo "[OK] $*"; }
  msg_info() { echo "[..] $*"; }
  msg_error() { echo "[!!] $*" >&2; }
fi

# Dedicated host user for install tree + compose (docker group). Not a hard
# sandbox — docker group ≈ root — but keeps secrets/files out of /root and
# away from other apps' trees on a shared LXC.
ensure_service_user() {
  local user="${SERVICE_USER:-jheffmedia}"
  local uid="${SERVICE_UID:-10001}"
  if ! id -u "$user" >/dev/null 2>&1; then
    useradd --system --uid "$uid" --home-dir "$INSTALL_DIR" --no-create-home \
      --shell /usr/sbin/nologin "$user"
    msg_ok "Created service user ${user} (uid ${uid})"
  else
    msg_ok "Using service user ${user}"
  fi
  if getent group docker >/dev/null 2>&1; then
    usermod -aG docker "$user" 2>/dev/null || true
  fi
}

harden_install_perms() {
  local user="${SERVICE_USER:-jheffmedia}"
  local env_file="${COMMON_DIR}/.env.production"
  local key_path="${INSTALL_DIR}/backend/src/secret.key"

  chown -R "${user}:${user}" "$INSTALL_DIR" 2>/dev/null || true
  if [[ -f "$env_file" ]]; then
    chmod 600 "$env_file"
    chown "${user}:${user}" "$env_file"
  fi
  if [[ -f "$key_path" ]]; then
    # Container UID 10001 must read the JWT mount; host operator owns the file.
    chmod 400 "$key_path"
    chown "${user}:${user}" "$key_path"
  fi
}

# Git ops as the install owner — avoids "dubious ownership" when root runs --update
# on a tree owned by jheffmedia (uid 10001). Prefer runuser over safe.directory.
install_git() {
  local user="${SERVICE_USER:-jheffmedia}"
  if [[ ! -d "${INSTALL_DIR}/.git" ]]; then
    return 1
  fi
  if [[ "$(id -un 2>/dev/null || true)" == "$user" ]]; then
    git -C "$INSTALL_DIR" "$@"
    return
  fi
  if [[ ${EUID:-0} -eq 0 ]] && id -u "$user" >/dev/null 2>&1; then
    if command -v runuser >/dev/null 2>&1; then
      runuser -u "$user" -- git -C "$INSTALL_DIR" "$@"
    else
      su -s /bin/sh "$user" -c "git -C $(printf '%q' "$INSTALL_DIR") $(printf '%q ' "$@")"
    fi
    return
  fi
  # Same-machine operator without matching uid: scoped exception (not --global).
  git -c "safe.directory=${INSTALL_DIR}" -C "$INSTALL_DIR" "$@"
}

# Upsert KEY=VALUE in an env file (no wipe of other keys). Value must not contain newlines.
upsert_env_var() {
  local file=$1 key=$2 value=$3 tmp
  [[ -f "$file" ]] || return 1
  tmp="$(mktemp)"
  # awk: portable vs macOS/GNU sed -i; match KEY= at line start only.
  awk -v k="$key" -v v="$value" '
    BEGIN { done = 0 }
    index($0, k "=") == 1 && !done { print k "=" v; done = 1; next }
    { print }
    END { if (!done) print k "=" v }
  ' "$file" >"$tmp" && mv "$tmp" "$file"
}

env_file_get() {
  local file=$1 key=$2
  [[ -f "$file" ]] || return 0
  grep -E "^${key}=" "$file" 2>/dev/null | head -1 | cut -d= -f2- || true
}

# Prefer existing env/file CORS; else PUBLIC_URL; else http://IP:PORT.
# Never clears a user-set YEW_FULLSTACK_CORS_ORIGINS.
ensure_cors_origins() {
  local env_file=${1:-}
  if [[ -n "${YEW_FULLSTACK_CORS_ORIGINS:-}" ]]; then
    export YEW_FULLSTACK_CORS_ORIGINS
    return 0
  fi
  local port="${HTTP_PORT:-8080}"
  if [[ -n "$env_file" ]]; then
    local existing
    existing="$(env_file_get "$env_file" YEW_FULLSTACK_CORS_ORIGINS)"
    if [[ -n "${existing// }" ]]; then
      YEW_FULLSTACK_CORS_ORIGINS="$existing"
      export YEW_FULLSTACK_CORS_ORIGINS
      return 0
    fi
    if [[ -z "${PUBLIC_URL:-}" ]]; then
      PUBLIC_URL="$(env_file_get "$env_file" PUBLIC_URL)"
    fi
    local p
    p="$(env_file_get "$env_file" HTTP_PORT)"
    [[ -n "$p" ]] && port="$p"
  fi
  if [[ -n "${PUBLIC_URL:-}" ]]; then
    YEW_FULLSTACK_CORS_ORIGINS="${PUBLIC_URL%/}"
    export YEW_FULLSTACK_CORS_ORIGINS
    return 0
  fi
  local ip
  ip="${LOCAL_IP:-}"
  [[ -z "$ip" ]] && ip="$(hostname -I 2>/dev/null | awk '{print $1}')"
  [[ -z "$ip" ]] && ip="127.0.0.1"
  YEW_FULLSTACK_CORS_ORIGINS="http://${ip}:${port}"
  export YEW_FULLSTACK_CORS_ORIGINS
}

# Persist BUILD_ID (always refresh) + CORS (only if missing) for compose/systemd.
# Empty compose ${YEW_FULLSTACK_BUILD_ID:-} would override the image ENV — keep .env filled.
persist_deploy_env() {
  local env_file=${1:-${COMMON_DIR}/.env.production}
  [[ -f "$env_file" ]] || return 0
  export_build_id
  ensure_cors_origins "$env_file"
  if [[ -n "${YEW_FULLSTACK_BUILD_ID:-}" ]]; then
    upsert_env_var "$env_file" YEW_FULLSTACK_BUILD_ID "$YEW_FULLSTACK_BUILD_ID"
  fi
  local cors_existing
  cors_existing="$(env_file_get "$env_file" YEW_FULLSTACK_CORS_ORIGINS)"
  if [[ -z "${cors_existing// }" && -n "${YEW_FULLSTACK_CORS_ORIGINS:-}" ]]; then
    upsert_env_var "$env_file" YEW_FULLSTACK_CORS_ORIGINS "$YEW_FULLSTACK_CORS_ORIGINS"
  fi
  if [[ -n "${PUBLIC_URL:-}" ]]; then
    local pub_existing
    pub_existing="$(env_file_get "$env_file" PUBLIC_URL)"
    if [[ -z "${pub_existing// }" ]]; then
      upsert_env_var "$env_file" PUBLIC_URL "${PUBLIC_URL%/}"
    fi
  fi
  chmod 600 "$env_file"
  # Re-export from file so compose interpolation sees them even if shell was empty.
  YEW_FULLSTACK_BUILD_ID="$(env_file_get "$env_file" YEW_FULLSTACK_BUILD_ID)"
  YEW_FULLSTACK_CORS_ORIGINS="$(env_file_get "$env_file" YEW_FULLSTACK_CORS_ORIGINS)"
  export YEW_FULLSTACK_BUILD_ID YEW_FULLSTACK_CORS_ORIGINS
}

write_env_file() {
  local env_file=$1
  export_build_id
  ensure_cors_origins ""
  cat >"$env_file" <<EOF
# Generated by install — edit and re-run: systemctl restart site
# Secrets live here only (chmod 600). Do not create /root/*.creds dumps.
HTTP_PORT=${HTTP_PORT}
HTTP_BIND=${HTTP_BIND:-0.0.0.0}
MONGO_PASSWORD=${MONGO_PASSWORD}
DB_CONNSTR=mongodb://root:${MONGO_PASSWORD}@db:27017/${DB_NAME}?authSource=admin
DB_NAME=${DB_NAME}
FEED_PROVIDER=${FEED_PROVIDER}
CONTACT_EMAIL=${CONTACT_EMAIL}
YEW_FULLSTACK_BUILD_ID=${YEW_FULLSTACK_BUILD_ID:-}
YEW_FULLSTACK_CORS_ORIGINS=${YEW_FULLSTACK_CORS_ORIGINS:-}
EOF

  if [[ -n "${PUBLIC_URL:-}" ]]; then
    printf 'PUBLIC_URL=%s\n' "${PUBLIC_URL%/}" >>"$env_file"
  fi

  if [[ "$FEED_PROVIDER" == "static" ]]; then
    # Compose env-files are not shell-parsed — do not use %q (it breaks JSON).
    printf 'FEED_STATIC_JSON=%s\n' "$FEED_STATIC_JSON" >>"$env_file"
  fi

  if [[ "$FEED_PROVIDER" == "immich" ]]; then
    cat >>"$env_file" <<EOF
IMMICH_URL=${IMMICH_URL}
IMMICH_API_KEY=${IMMICH_API_KEY}
IMMICH_ALBUM_ID=${IMMICH_ALBUM_ID}
EOF
    [[ -n "${IMMICH_SHARED_KEY:-}" ]] && \
      printf 'IMMICH_SHARED_KEY=%s\n' "$IMMICH_SHARED_KEY" >>"$env_file"
  fi

  if [[ -n "${ADMIN_EMAILS:-}" ]]; then
    printf 'ADMIN_EMAILS=%s\n' "$ADMIN_EMAILS" >>"$env_file"
  fi

  chmod 600 "$env_file"
}

# Legacy plaintext dumps (older installers). Secrets live in .env.production only.
legacy_creds_paths() {
  local -a paths=(
    /root/site-install.creds
    "${HOME}/site-install.creds"
    "${INSTALL_DIR}/site-install.creds"
    "${INSTALL_DIR}/install/site-install.creds"
  )
  # Distinct from HOME when sudo/root vs operator home.
  if [[ -n "${SUDO_USER:-}" ]]; then
    local sudo_home
    sudo_home="$(getent passwd "$SUDO_USER" 2>/dev/null | cut -d: -f6 || true)"
    [[ -n "$sudo_home" ]] && paths+=("${sudo_home}/site-install.creds")
  fi
  printf '%s\n' "${paths[@]}" | awk 'NF && !seen[$0]++'
}

remove_legacy_creds_file() {
  local f
  while IFS= read -r f; do
    [[ -z "$f" || ! -f "$f" ]] && continue
    rm -f "$f"
    msg_ok "Removed legacy ${f} (secrets live in .env.production)"
  done < <(legacy_creds_paths)
}

# Project-scoped docker tidy — never docker system prune -a.
# Exited jheffmedia-* containers only. App images: --reinstall --purge (no labels).
cleanup_project_docker() {
  local dry=${1:-0}
  local bin="${CONTAINER_BIN:-docker}"
  command -v "$bin" >/dev/null 2>&1 || {
    msg_info "Docker not available — skip project prune"
    return 0
  }

  local ids
  ids="$("$bin" ps -aq -f status=exited -f name=jheffmedia 2>/dev/null || true)"
  if [[ -n "$ids" ]]; then
    if [[ "$dry" -eq 1 ]]; then
      msg_info "dry-run: would remove exited containers: $(echo "$ids" | tr '\n' ' ')"
    else
      # shellcheck disable=SC2086
      "$bin" rm $ids >/dev/null
      msg_ok "Removed exited jheffmedia containers"
    fi
  else
    msg_info "No exited jheffmedia containers"
  fi
}

ensure_secret_key() {
  local key_path=$1
  if [[ ! -f "$key_path" ]]; then
    openssl rand -out "$key_path" 32
    msg_ok "Generated JWT secret key"
  else
    msg_ok "Reusing existing JWT secret key"
  fi
  chmod 400 "$key_path"
}

install_systemd() {
  local unit=/etc/systemd/system/site.service
  local bin after requires
  local user="${SERVICE_USER:-jheffmedia}"
  bin="$(command -v "$CONTAINER_BIN")"
  after="network-online.target"
  requires=""
  if [[ "$CONTAINER_BIN" == docker ]]; then
    after="docker.service network-online.target"
    requires="Requires=docker.service"
  fi
  cat >"$unit" <<EOF
[Unit]
Description=jheff.media container stack
After=${after}
Wants=network-online.target
${requires}

[Service]
Type=oneshot
RemainAfterExit=yes
User=${user}
Group=${user}
SupplementaryGroups=docker
WorkingDirectory=${COMMON_DIR}
Environment=HTTP_PORT=${HTTP_PORT}
Environment=HTTP_BIND=${HTTP_BIND:-0.0.0.0}
# docker.sock access via SupplementaryGroups=docker — never mount the socket into the app container.
ExecStart=${bin} compose -f docker-compose.yml --env-file .env.production up -d --remove-orphans
ExecStop=${bin} compose -f docker-compose.yml --env-file .env.production down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload
  systemctl enable -q --now site

  # Host CLI: jheffmedia --update | --cleanup | --reinstall | --status | --restart | --postinstall
  local cli="${INSTALL_DIR}/install/common/bin/jheffmedia"
  chmod +x "$cli" "${INSTALL_DIR}/install/update.sh" "${INSTALL_DIR}/install/postinstall.sh" 2>/dev/null || true
  ln -sfn "$cli" /usr/local/bin/jheffmedia
  printf '%s\n' '#!/bin/sh' 'exec /usr/local/bin/jheffmedia --update "$@"' >/usr/local/bin/site-update
  chmod +x /usr/local/bin/site-update
}

# Optional release identity for Admin → Dev / diagnostics (override by exporting first).
export_build_id() {
  if [[ -z "${YEW_FULLSTACK_BUILD_ID:-}" && -d "${INSTALL_DIR}/.git" ]]; then
    YEW_FULLSTACK_BUILD_ID="$(install_git rev-parse --short HEAD 2>/dev/null || true)"
    # install_git can fail (ownership); still try a scoped git read.
    if [[ -z "$YEW_FULLSTACK_BUILD_ID" ]]; then
      YEW_FULLSTACK_BUILD_ID="$(
        git -c "safe.directory=${INSTALL_DIR}" -C "$INSTALL_DIR" rev-parse --short HEAD 2>/dev/null || true
      )"
    fi
  fi
  export YEW_FULLSTACK_BUILD_ID="${YEW_FULLSTACK_BUILD_ID:-}"
}

# Quiet compose build with heartbeats. VERBOSE=yes streams full output.
# Extra args (e.g. --no-cache) are passed to `compose build`.
# ponytail: poll+tail beats a TTY progress parser; raise BUILD_HEARTBEAT_SECS if noisy.
compose_build() {
  # Write BUILD_ID (+ CORS if missing) into .env.production before compose interpolates.
  persist_deploy_env "${COMMON_DIR}/.env.production"
  # BuildKit + cache mounts in Dockerfile (cargo registry/git/target). Do not default --no-cache.
  export DOCKER_BUILDKIT=1
  export COMPOSE_DOCKER_CLI_BUILD=1
  local -a build_cmd=(
    "$CONTAINER_BIN" compose -f docker-compose.yml --env-file .env.production build "$@"
  )
  local interval="${BUILD_HEARTBEAT_SECS:-20}"

  if [[ "${VERBOSE:-no}" == "yes" ]]; then
    "${build_cmd[@]}"
    return
  fi

  local log pid rc=0 elapsed=0 line
  log="$(mktemp "${TMPDIR:-/tmp}/compose-build.XXXXXX")"
  "${build_cmd[@]}" >"$log" 2>&1 &
  pid=$!

  while kill -0 "$pid" 2>/dev/null; do
    sleep 1
    elapsed=$((elapsed + 1))
    ((elapsed % interval == 0)) || continue
    kill -0 "$pid" 2>/dev/null || break
    line="$(
      tr '\r' '\n' <"$log" 2>/dev/null \
        | sed 's/\x1b\[[0-9;]*[a-zA-Z]//g' \
        | grep -E '[[:alnum:]]' \
        | tail -1 \
        | cut -c1-100 || true
    )" || true
    # Newline so community-scripts spinner keeps the current line; scrollback shows progress.
    printf '\n[..] still building… %dm%02ds%s\n' \
      "$((elapsed / 60))" "$((elapsed % 60))" \
      "${line:+ — ${line}}" >&2
  done

  wait "$pid" || rc=$?
  if [[ $rc -ne 0 ]]; then
    echo "[!!] compose build failed — last log lines:" >&2
    tail -n 50 "$log" >&2 || true
  fi
  rm -f "$log"
  return "$rc"
}

deploy_stack() {
  local compose_dir="${COMMON_DIR}"
  local env_file="${compose_dir}/.env.production"

  ensure_service_user
  ensure_secret_key "${INSTALL_DIR}/backend/src/secret.key"
  write_env_file "$env_file"
  remove_legacy_creds_file
  harden_install_perms

  msg_info "Building application image (Rust + Trunk — may take several minutes)"
  cd "$compose_dir"
  export HTTP_PORT HTTP_BIND
  # shellcheck disable=SC1091
  source "${INSTALL_DIR}/scripts/lib/container.sh"
  require_container
  compose_build
  msg_ok "Built application image"

  msg_info "Starting stack"
  export_build_id
  $STD "$CONTAINER_BIN" compose -f docker-compose.yml --env-file .env.production up -d --remove-orphans
  msg_ok "Stack running on ${HTTP_BIND:-0.0.0.0}:${HTTP_PORT}"

  install_systemd
  echo "${BRANCH}" >"${compose_dir}/.install-version"
  # Re-apply ownership after build artifacts / version marker.
  harden_install_perms

  # Fresh install: surface admin signup path once (secrets stay in .env.production).
  if [[ -x "${INSTALL_DIR}/install/postinstall.sh" ]]; then
    INSTALL_DIR="$INSTALL_DIR" "${INSTALL_DIR}/install/postinstall.sh" --fresh
  fi
}

update_stack() {
  local compose_dir="${COMMON_DIR}"
  [[ -d "$INSTALL_DIR/.git" ]] || return 1

  ensure_service_user
  msg_info "Pulling latest ${BRANCH} from origin"
  # HTTPS GitHub clone (TLS). Day-2 updates use this tree — prefer jheffmedia --update
  # over curl|bash so you are not re-fetching install scripts from raw CDN each time.
  install_git fetch origin "$BRANCH"
  install_git checkout "$BRANCH"
  install_git pull --ff-only origin "$BRANCH"
  msg_ok "Repository updated"

  cd "$compose_dir"
  export HTTP_PORT="${HTTP_PORT:-8080}"
  export HTTP_BIND="${HTTP_BIND:-0.0.0.0}"
  # shellcheck disable=SC1091
  source "${INSTALL_DIR}/scripts/lib/container.sh"
  require_container
  msg_info "Building application image (Rust + Trunk — may take several minutes)"
  compose_build
  msg_ok "Built application image"
  msg_info "Starting stack"
  export_build_id
  $STD "$CONTAINER_BIN" compose -f docker-compose.yml --env-file .env.production up -d --remove-orphans
  # Refresh unit so User=jheffmedia lands on older installs.
  if [[ ${EUID:-0} -eq 0 ]]; then
    install_systemd
  else
    systemctl restart site 2>/dev/null || true
  fi
  echo "${BRANCH}" >"${compose_dir}/.install-version"
  harden_install_perms
  remove_legacy_creds_file
  msg_ok "Stack updated"
}

clone_repo() {
  msg_info "Cloning repository (${BRANCH})"
  if [[ -d "${INSTALL_DIR}/.git" ]]; then
    msg_ok "Using existing clone at ${INSTALL_DIR}"
  else
    # Clone over HTTPS (TLS to github.com). Depth-1 pin to BRANCH.
    $STD git clone --branch "$BRANCH" --depth 1 "$REPO" "$INSTALL_DIR"
    # Mark directory safe for any later root-owned tooling; primary path is install_git.
    git -C "$INSTALL_DIR" config --local --add safe.directory "$INSTALL_DIR" 2>/dev/null || true
    msg_ok "Cloned repository"
  fi
  COMMON_DIR="${INSTALL_DIR}/install/common"
}
