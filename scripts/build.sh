#!/usr/bin/env bash

WORKDIR="$( cd "$(dirname "$0")"/.. ; pwd -P )"

rm -rf "$WORKDIR/frontend/dist"

docker build -t yew-fullstack/application -t yew-fullstack/application:latest "$WORKDIR"
