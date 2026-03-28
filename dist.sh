#!/usr/bin/env sh
set -eu

# Build release binary and copy to bin/ (Linux/FreeBSD).
cargo build --release -p fpas-cli
mkdir -p bin
cp target/release/fpas bin/fpas
chmod +x bin/fpas
echo "Built: bin/fpas"
