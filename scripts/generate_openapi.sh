#!/usr/bin/env bash
set -euo pipefail

# Generates OpenAPI spec from signia-api (if the project uses utoipa/axum integration),
# or copies the committed docs/api/openapi.yaml into the crate output.
#
# This script is best-effort: the repository may ship the OpenAPI spec as a source file.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

SRC="docs/api/openapi.yaml"
OUT="docs/api/openapi.generated.yaml"

if [ -f "${SRC}" ]; then
  cp "${SRC}" "${OUT}"
  echo "generated: ${OUT} (copied from ${SRC})"
else
  echo "openapi source not found at ${SRC}"
  exit 1
fi
