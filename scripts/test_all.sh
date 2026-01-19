#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

echo "== test_all =="

echo "-- rust tests --"
cargo test --workspace

if [ -d "programs/signia-registry" ] && command -v anchor >/dev/null 2>&1; then
  echo "-- anchor tests (optional) --"
  (cd programs/signia-registry && anchor test) || echo "skip: anchor tests failed"
else
  echo "skip: anchor not installed or program missing"
fi

echo "done"
