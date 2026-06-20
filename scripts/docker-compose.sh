#!/usr/bin/env bash
# Resolve docker compose CLI (v2 plugin vs legacy docker-compose).
if docker compose version >/dev/null 2>&1; then
  exec docker compose "$@"
elif command -v docker-compose >/dev/null 2>&1; then
  exec docker-compose "$@"
else
  echo "error: neither 'docker compose' nor 'docker-compose' found. Install Docker Desktop or docker-compose." >&2
  exit 1
fi
