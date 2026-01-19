#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

if ! command -v anchor >/dev/null 2>&1; then
  echo "anchor not found."
  echo "Install Anchor: https://www.anchor-lang.com/docs/installation"
  exit 1
fi

echo "== anchor_build =="
(cd programs/signia-registry && anchor build)
echo "done"
