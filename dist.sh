#!/usr/bin/env sh
set -eu
CDPATH= cd -- "$(dirname -- "$0")"

# Build release binaries and bundled standard-library sources in bin/ (Linux).
cargo build --release -p fpas-cli
cargo run --release -p fpas-build --example precompile_stdlib -- target/release/lib bin/lib
mkdir -p bin
cp target/release/fpas bin/fpas
cp target/release/fpas-runner bin/fpas-runner
chmod +x bin/fpas bin/fpas-runner
echo "Built: bin/fpas, bin/fpas-runner, and bin/lib"
