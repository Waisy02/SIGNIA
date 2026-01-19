#!/usr/bin/env bash
set -euo pipefail

if [ "${1:-}" = "" ]; then
  echo "no command provided"
  exit 1
fi

exec "$@"
