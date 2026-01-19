#!/usr/bin/env bash
set -euo pipefail

# SIGNIA bootstrap script
# Installs/validates required toolchains and common dependencies.
#
# This script is intentionally conservative and avoids sudo.
# It prints actionable instructions if something is missing.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

has() { command -v "$1" >/dev/null 2>&1; }

echo "== SIGNIA bootstrap =="
echo "repo: ${ROOT_DIR}"

# ---- Rust ----
if ! has rustup; then
  echo "rustup not found."
  echo "Install rustup: https://rustup.rs"
  exit 1
fi

echo "checking Rust toolchain..."
rustup show >/dev/null 2>&1 || true
rustup toolchain install stable -q || true
rustup default stable >/dev/null 2>&1 || true
rustup component add clippy rustfmt >/dev/null 2>&1 || true

echo "Rust OK: $(rustc --version)"

# ---- Node ----
if ! has node; then
  echo "node not found."
  echo "Install Node.js >= 18 (recommend 20): https://nodejs.org"
  exit 1
fi

NODE_MAJOR="$(node -p "process.versions.node.split('.')[0]")"
if [ "${NODE_MAJOR}" -lt 18 ]; then
  echo "Node.js >= 18 required. Found: $(node --version)"
  exit 1
fi
echo "Node OK: $(node --version)"

if has corepack; then
  corepack enable >/dev/null 2>&1 || true
fi

# ---- Solana ----
if ! has solana; then
  echo "solana CLI not found."
  echo "Install: https://docs.solana.com/cli/install-solana-cli-tools"
  echo "Proceeding without Solana tooling."
else
  echo "Solana OK: $(solana --version)"
fi

# ---- Anchor ----
if ! has anchor; then
  echo "anchor not found."
  echo "Install Anchor: https://www.anchor-lang.com/docs/installation"
  echo "Proceeding without Anchor tooling."
else
  echo "Anchor OK: $(anchor --version)"
fi

# ---- Docker ----
if ! has docker; then
  echo "docker not found (optional for e2e/local stack)."
else
  echo "Docker OK: $(docker --version)"
fi

echo
echo "Bootstrap complete."
echo "Next:"
echo "  bash scripts/build_all.sh"
echo "  bash scripts/test_all.sh"
