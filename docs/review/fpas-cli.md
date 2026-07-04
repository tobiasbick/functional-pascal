# fpas-cli Review

## Summary

`fpas-cli` is the broadest user-facing crate. Package Clippy now passes after the `fpas-vm` lint blockers were resolved on 2026-07-04. The main remaining concern is structural size in CLI parsing and test-runner modules.

## Findings

### Resolved: CLI Clippy was blocked by VM dependency warnings

Evidence: `cargo clippy -p fpas-cli --all-targets -- -D warnings` previously failed while checking `fpas-vm`, specifically `crates/fpas-vm/src/vm/shared.rs:51` (`clippy::derivable_impls` on `TuiState`).

Impact: CLI-only changes could not rely on strict package Clippy as a clean signal.

Resolution: `fpas-vm` lint blockers were fixed on 2026-07-04. `cargo clippy -p fpas-cli --all-targets -- -D warnings` now passes.

### Medium: CLI argument parsing and test runner files are oversized

Evidence: `crates/fpas-cli/src/cli_input.rs` has 446 lines, `crates/fpas-cli/src/cli_test/mod.rs` has 672 lines, and `crates/fpas-cli/src/cli_test/run.rs` has 417 lines. `crates/fpas-cli/src/project/tests/imports.rs` is also close at 417 lines.

Impact: `cli_input.rs` mixes help text, run/check/test/fmt config models, and parsing. `cli_test/mod.rs` mixes orchestration, output, filtering, sequential execution, parallel execution, and failure accounting. This makes user-facing CLI behavior harder to change safely.

Next step: split `cli_input.rs` into submodules by command (`run_check`, `test`, `fmt`, `help`) and split `cli_test/mod.rs` orchestration into result accounting, sequential runner, parallel runner, and output/reporting glue.

## Verification

- `cargo clippy -p fpas-cli --all-targets -- -D warnings` passed after the `fpas-vm` fix.
