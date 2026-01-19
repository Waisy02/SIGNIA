#!/usr/bin/env bash
set -euo pipefail

# Starts a local solana-test-validator configured for Anchor.
# This is optional and used for local development.

if ! command -v solana-test-validator >/dev/null 2>&1; then
  echo "solana-test-validator not found. Install Solana CLI."
  exit 1
fi

LEDGER_DIR="${LEDGER_DIR:-.solana/test-ledger}"
RESET="${RESET:-1}"

ARGS=("--ledger" "${LEDGER_DIR}" "--rpc-port" "8899" "--faucet-port" "9900")
if [ "${RESET}" = "1" ]; then
  ARGS+=("--reset")
fi

echo "starting solana-test-validator..."
echo "ledger: ${LEDGER_DIR}"
solana-test-validator "${ARGS[@]}"
