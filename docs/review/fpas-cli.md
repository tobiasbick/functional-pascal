# fpas-cli Review

## Summary

`fpas-cli` is the broadest user-facing crate. Package Clippy passes. CLI argument parsing, test-runner code, integration tests, and project import regression tests are now split into focused modules. No `fpas-cli` Rust source file currently exceeds the repository's 400-line split threshold.

## Findings

### Resolved: CLI Clippy was blocked by VM dependency warnings

Evidence: `cargo clippy -p fpas-cli --all-targets -- -D warnings` previously failed while checking `fpas-vm`, specifically `crates/fpas-vm/src/vm/shared.rs:51` (`clippy::derivable_impls` on `TuiState`).

Impact: CLI-only changes could not rely on strict package Clippy as a clean signal.

Resolution: `fpas-vm` lint blockers were fixed on 2026-07-04. `cargo clippy -p fpas-cli --all-targets -- -D warnings` now passes.

### Resolved: `cli_input.rs` mixed help text, types, discovery, and argv parsing

Evidence: resolved on 2026-07-04. `crates/fpas-cli/src/cli_input.rs` had 446 lines. CLI help text now lives in `cli_input/help.rs` (22 lines), configuration types in `cli_input/types.rs` (50 lines), mode and argv helpers in `cli_input/mode.rs` (46 lines), project discovery in `cli_input/discovery.rs` (100 lines), and `resolve_cli_config` in `cli_input/mod.rs` (255 lines).

Impact: Run/check/test/fmt argument models, discovery, and parsing were coupled in one file.

Resolution: split by command concern while keeping `resolve_cli_config` as the single public entry.

### Resolved: `cli_test/mod.rs` mixed orchestration, runners, and integration tests

Evidence: resolved on 2026-07-04. `crates/fpas-cli/src/cli_test/mod.rs` had 672 lines. Test discovery and `test_cli` entry now live in `cli_test/mod.rs` (78 lines), sequential and parallel orchestration in `cli_test/runner.rs` (112 lines).

Impact: Test-runner control flow and a large integration-test suite were coupled in one module root.

Resolution: move runner glue to `runner.rs`.

### Resolved: `cli_test/run.rs` mixed load, hook execution, and VM program runs

Evidence: resolved on 2026-07-04. `crates/fpas-cli/src/cli_test/run.rs` had 417 lines. Program loading and script application now live in `cli_test/run/load.rs` (94 lines), setup/teardown hook execution in `cli_test/run/hook_exec.rs` (82 lines), compile/run/golden comparison in `cli_test/run/program.rs` (198 lines), and `run_single_test` orchestration in `cli_test/run/mod.rs` (88 lines).

Impact: Compile/link, hook programs, script setup, VM execution, and golden comparison were coupled in one file.

Resolution: split by execution phase while keeping `run_single_test` as the single entry.

### Resolved: `cli_test/tests.rs` mixed discovery, golden, timeout, and skip integration tests

Evidence: resolved on 2026-07-04. `crates/fpas-cli/src/cli_test/tests.rs` had 490 lines. Integration tests now live under `cli_test/tests/` in `discovery.rs` (180 lines), `golden.rs` (132 lines), `skip.rs` (93 lines), `reporting.rs` (35 lines), `run_timeout.rs` (33 lines), and `validation.rs` (33 lines).

Impact: Discovery/filter/jobs behavior, golden sidecars, timeout handling, and skip/strict reporting were coupled in one test module.

Resolution: split by test theme consistent with `tests/tui/` layout in the FPAS regression suite.

### Resolved: `project/tests/imports.rs` mixed import resolution, graph, and error scenarios

Evidence: resolved on 2026-07-04. `crates/fpas-cli/src/project/tests/imports.rs` had 417 lines. Import regression tests now live under `project/tests/imports/` in `graph.rs` (184 lines), `short_names.rs` (107 lines), `errors.rs` (60 lines), `sources.rs` (47 lines), `visibility.rs` (44 lines), and `uses.rs` (34 lines).

Impact: Short-name resolution, dependency graph behavior, visibility, and linker error messages were coupled in one test module.

Resolution: split by import/linking scenario.

## Verification

- `cargo clippy -p fpas-cli --all-targets -- -D warnings` passed after the import-test split.
- `cargo test -p fpas-cli` passed (287 tests).
