# fpas-cli Review

## Summary

`fpas-cli` is the broadest user-facing crate. It could not be Clippy-verified independently because `fpas-vm` fails under strict linting.

## Findings

### Medium: CLI Clippy is blocked by VM dependency warnings

Evidence: `cargo clippy -p fpas-cli --all-targets -- -D warnings` failed while checking `fpas-vm`, specifically `crates/fpas-vm/src/vm/shared.rs:51`.

Impact: CLI-only changes cannot currently rely on strict package Clippy as a clean signal. This matters because the CLI owns high-level workflows such as `run`, `check`, `test`, and `fmt`.

Suggested fix: Fix the `fpas-vm` lint blockers first, then rerun CLI package Clippy.

### Medium: CLI argument parsing and test runner files are oversized

Evidence: `crates/fpas-cli/src/cli_input.rs` has 446 lines, `crates/fpas-cli/src/cli_test/mod.rs` has 672 lines, and `crates/fpas-cli/src/cli_test/run.rs` has 417 lines.

Impact: `cli_input.rs` mixes help text, run/check/test/fmt config models, and parsing. `cli_test/mod.rs` mixes orchestration, output, filtering, sequential execution, parallel execution, and failure accounting. This makes user-facing CLI behavior harder to change safely.

Suggested fix: Split `cli_input.rs` into submodules by command (`run_check`, `test`, `fmt`, `help`) and split `cli_test/mod.rs` orchestration into result accounting, sequential runner, parallel runner, and output/reporting glue.

## Verification

- `cargo clippy -p fpas-cli --all-targets -- -D warnings` failed because `fpas-vm` fails.

