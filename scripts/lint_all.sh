#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

echo "== lint_all =="

echo "-- rustfmt --"
cargo fmt --all -- --check

echo "-- clippy --"
cargo clippy --workspace --all-targets -- -D warnings

if [ -d "console/web" ]; then
  echo "-- eslint console/web (best-effort) --"
  (cd console/web && npm install && npm run lint) || echo "skip: console/web lint failed or not configured"
fi

if [ -d "console/interface" ]; then
  echo "-- eslint console/interface (best-effort) --"
  (cd console/interface && npm install && npm run lint) || echo "skip: console/interface lint failed or not configured"
fi

echo "done"
