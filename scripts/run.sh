#!/usr/bin/env bash

WORKDIR="$( cd "$(dirname "$0")"/.. ; pwd -P )" # get the project dir
COMPOSE="$WORKDIR/scripts/docker-compose.sh"

cd "$WORKDIR/scripts"
"$COMPOSE" \
    -f "$WORKDIR/scripts/docker-compose.yml" \
    -p yew-fullstack \
    up \
    --force-recreate --remove-orphans \
    --exit-code-from application \
    --ignore-pull-failures
