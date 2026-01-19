#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN="${ROOT_DIR}/target/debug/signia"

if [ ! -x "${BIN}" ]; then
  echo "signia CLI not found at ${BIN}"
  echo "Build it from repo root: cargo build -p signia-cli"
  exit 1
fi

INPUT_FILE="${ROOT_DIR}/examples/repo-github/input.txt"
OUT_DIR="${ROOT_DIR}/examples/repo-github/out"
mkdir -p "${OUT_DIR}"

echo "== compile =="
"${BIN}" compile "${INPUT_FILE}" --type repo --out "${OUT_DIR}"

echo "== verify =="
"${BIN}" verify "${OUT_DIR}/schema.json" "${OUT_DIR}/proof.json" --manifest "${OUT_DIR}/manifest.json"

echo "== publish (optional) =="
if [ "${SIGNIA_PUBLISH_DEVNET:-}" = "1" ]; then
  "${BIN}" publish "${OUT_DIR}/schema.json" --devnet
else
  echo "skip publish (set SIGNIA_PUBLISH_DEVNET=1 to enable)"
fi

echo "done"
