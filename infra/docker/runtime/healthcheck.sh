#!/usr/bin/env bash
set -euo pipefail

URL="${1:-}"
if [ -z "${URL}" ]; then
  echo "usage: healthcheck.sh <url>"
  exit 2
fi

curl -fsS "${URL}" >/dev/null
