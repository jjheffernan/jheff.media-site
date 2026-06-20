#!/usr/bin/env bash

WORKDIR="$( cd "$(dirname "$0")"/.. ; pwd -P )"
COMPOSE="$WORKDIR/scripts/docker-compose.sh"

cd "$WORKDIR/scripts"

"$COMPOSE" \
    -f "$WORKDIR/scripts/docker-compose.dev.yml" \
    -p yew-fullstack-dev \
    up \
    --force-recreate --remove-orphans \
    --exit-code-from backend
