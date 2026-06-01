# Runs the curated non-interactive example allowlist (see crates/fpas-cli/src/main_tests/examples.rs).
# Safe for CI and agents — does not start interactive TUI/graph demos.
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
cargo test -p fpas-cli non_interactive_examples_run_successfully --
