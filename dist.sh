#!/usr/bin/env sh
set -eu
CDPATH= cd -- "$(dirname -- "$0")"

# Build release binaries and bundled standard-library sources in bin/ (Linux).
cargo build --locked --release -p fpas-cli -p fpas-lsp -p fpas-runner
./target/release/fpas stdlib stage --source target/release/lib --output bin/lib
mkdir -p bin
cp target/release/fpas bin/fpas
cp target/release/fpas-runner bin/fpas-runner
cp target/release/fpas-lsp bin/fpas-lsp
chmod +x bin/fpas bin/fpas-runner bin/fpas-lsp
echo "Built: bin/fpas, bin/fpas-runner, bin/fpas-lsp, and bin/lib"
