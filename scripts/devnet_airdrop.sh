#!/usr/bin/env bash
set -euo pipefail

# Requests an airdrop on Solana devnet for the currently configured keypair.

if ! command -v solana >/dev/null 2>&1; then
  echo "solana CLI not found."
  exit 1
fi

AMOUNT="${1:-2}"
solana config set --url https://api.devnet.solana.com >/dev/null
PUBKEY="$(solana address)"
echo "requesting airdrop: ${AMOUNT} SOL to ${PUBKEY}"
solana airdrop "${AMOUNT}" "${PUBKEY}"
solana balance
