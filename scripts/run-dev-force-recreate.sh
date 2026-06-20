#!/usr/bin/env bash

WORKDIR="$( cd "$(dirname "$0")"/.. ; pwd -P )" # get the project dir
COMPOSE="$WORKDIR/scripts/docker-compose.sh"

cd "$WORKDIR/frontend"

rm -rf node_modules

yarn install

cd "$WORKDIR/scripts"

"$COMPOSE" \
    -f "$WORKDIR/scripts/docker-compose.dev.yml" \
    -p yew-fullstack-dev \
    up \
    --force-recreate --remove-orphans --build \
    --exit-code-from backend
