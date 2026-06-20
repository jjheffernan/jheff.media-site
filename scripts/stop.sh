#!/usr/bin/env bash

WORKDIR="$( cd "$(dirname "$0")"/.. ; pwd -P )" # get the project dir
COMPOSE="$WORKDIR/scripts/docker-compose.sh"

"$COMPOSE" \
    -f "$WORKDIR/scripts/docker-compose.yml" \
    down -v --rmi all --remove-orphans

cd "$WORKDIR/scripts"
"$COMPOSE" \
    -f "$WORKDIR/scripts/docker-compose.yml" \
    -p yew-fullstack kill

"$COMPOSE" \
    -f "$WORKDIR/scripts/docker-compose.yml" \
    -p yew-fullstack rm -f
