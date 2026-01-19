#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

echo "== build_all =="

echo "-- building Rust workspace --"
cargo build --workspace

if [ -d "console/web" ]; then
  echo "-- building console/web --"
  (cd console/web && npm install && npm run build)
fi

if [ -d "console/interface" ]; then
  echo "-- building console/interface --"
  (cd console/interface && npm install && npm run build)
fi

if [ -d "sdk/ts" ]; then
  echo "-- building sdk/ts --"
  (cd sdk/ts && npm install && npm run build)
fi

echo "done"
