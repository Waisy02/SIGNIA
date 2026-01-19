#!/usr/bin/env bash
set -euo pipefail

# Local release helper.
# This script does NOT publish automatically. It builds artifacts and prints next steps.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

VERSION="${1:-}"
if [ -z "${VERSION}" ]; then
  echo "usage: bash scripts/release.sh <version>"
  exit 1
fi

echo "== release ${VERSION} =="

echo "-- build workspace --"
cargo build --workspace --release

echo "-- build ts sdk --"
if [ -d "sdk/ts" ]; then
  (cd sdk/ts && npm install && npm run build)
fi

echo "-- build console --"
if [ -d "console/web" ]; then
  (cd console/web && npm install && npm run build)
fi
if [ -d "console/interface" ]; then
  (cd console/interface && npm install && npm run build)
fi

echo
echo "Artifacts are built."
echo "Next steps (manual):"
echo "  - tag git: git tag v${VERSION} && git push --tags"
echo "  - create GitHub release (attach artifacts if needed)"
echo "  - publish npm package from sdk/ts"
echo "  - publish python package from sdk/python"
