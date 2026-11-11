#!/usr/bin/env bash
# Create/update a bare git mirror for Coolify (no GitHub access required on Coolify).
#
# On a machine that can reach the source (once):
#   MIRROR_DIR=/srv/git/jheff.media-site.git bash install/coolify/mirror.sh
#
# Coolify Private Repository URL examples:
#   git@mirror-host:/srv/git/jheff.media-site.git
#   ssh://git@mirror-host/srv/git/jheff.media-site.git
#   file:///srv/git/jheff.media-site.git   # same host as Coolify

set -euo pipefail

SOURCE="${SOURCE:-https://github.com/jjheffernan/jheff.media-site.git}"
MIRROR_DIR="${MIRROR_DIR:-/srv/git/jheff.media-site.git}"

mkdir -p "$(dirname "$MIRROR_DIR")"

if [[ -d "$MIRROR_DIR" ]]; then
  echo "Updating mirror at ${MIRROR_DIR}"
  git -C "$MIRROR_DIR" remote set-url origin "$SOURCE"
  git -C "$MIRROR_DIR" fetch --all --prune
else
  echo "Cloning mirror ${SOURCE} → ${MIRROR_DIR}"
  git clone --mirror "$SOURCE" "$MIRROR_DIR"
fi

# Enough for dumb HTTP if you put a web server in front of the bare repo.
git -C "$MIRROR_DIR" update-server-info

echo "OK  Coolify → ${MIRROR_DIR}  (branch: main)"
echo "    Dockerfile: Dockerfile"
echo "    Port: 8080"
echo "    Env file: install/coolify/env.example"
