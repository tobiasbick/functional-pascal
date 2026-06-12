#!/usr/bin/env bash
# Formats all `.fpas` sources under examples/, tests/, and apps/ (skips target/).
# Check only: scripts/format-fpas-sources.sh --check
# List dirty paths: scripts/format-fpas-sources.sh --check --list
set -euo pipefail
cd "$(dirname "$0")/.."

args=()
if [[ "${1:-}" == "--check" ]]; then
  args+=(--check)
  shift
fi
if [[ "${1:-}" == "--list" ]]; then
  args+=(--list)
  shift
fi

cargo run -q -p fpas-cli -- fmt "${args[@]}" examples tests apps
