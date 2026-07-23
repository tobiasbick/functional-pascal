#!/usr/bin/env sh
set -eu

# Build release binary and bundled standard-library sources in bin/ (Linux/FreeBSD).
cargo build --release -p fpas-cli
cargo run --release -p fpas-build --example precompile_stdlib -- target/release/lib
mkdir -p bin
cp target/release/fpas bin/fpas
mkdir -p bin/lib
cp -R target/release/lib/. bin/lib/
chmod +x bin/fpas
echo "Built: bin/fpas and bin/lib"
