# fpas-vm Review

## Summary

This crate had the highest-priority findings because it blocked workspace Clippy and downstream package Clippy for `fpas-compiler` and `fpas-cli`. The lint blockers were resolved on 2026-07-04.

## Findings

### Resolved: Workspace Clippy failed on a derivable `Default` implementation

Evidence: `cargo clippy --workspace --all-targets -- -D warnings` reports `clippy::derivable_impls` at `crates/fpas-vm/src/vm/shared.rs:51`. `TuiState` manually initializes only default values.

Impact: Strict workspace lint verification fails before later crates can be cleanly checked. This is not a runtime bug, but it blocks the normal review gate.

Resolution: `TuiState` now derives `Default`.

### Resolved: Workspace Clippy failed on dead VM test helpers

Evidence: `cargo clippy --workspace --all-targets -- -D warnings` reports unused helpers at `crates/fpas-vm/src/tests/helpers.rs:35`, `crates/fpas-vm/src/tests/helpers.rs:59`, `crates/fpas-vm/src/tests/helpers.rs:80`, `crates/fpas-vm/src/tests/helpers.rs:90`, and `crates/fpas-vm/src/tests/helpers.rs:102`.

Impact: Dead test helpers make the test API look broader than it is and block strict all-target linting.

Resolution: The unused helpers and their imports were removed.

### Resolved: VM execution and TUI bridge files exceeded the structure threshold

Evidence: resolved for `fpas-vm`; a recursive scan under `crates/fpas-vm/src` now has no Rust files above 400 lines. `crates/fpas-vm/src/vm/shared.rs` was split into `shared.rs` (260 lines), `shared/tui.rs` (173 lines), and `shared/graph.rs` (67 lines). `crates/fpas-vm/src/vm/execute/io/tui/controls.rs` moved control creation to `control_create.rs`, leaving `controls.rs` (287 lines) and `control_create.rs` (283 lines). `crates/fpas-vm/src/vm/execute/io/tui/tv_run.rs` moved view construction to `tv_views.rs`, leaving `tv_run.rs` (198 lines) and `tv_views.rs` (338 lines). Additional cleanup split `handles.rs` (370 lines) from `handle_records.rs` (64 lines), split `console.rs` (268 lines) from `console_records.rs` (182 lines), and split graph drawing VM tests from `graph_vm.rs` (221 lines) into `graph_vm/draw.rs` (200 lines).

Impact: TUI creation, handle storage, reconciliation, run-loop integration, and shared runtime state were tightly coupled. The split reduces the risk of changing unrelated TUI behavior while adding a single control or event path.

Next step: continue with `fpas-cli` structure cleanup per [fpas-cli.md](fpas-cli.md).

## Verification

- `cargo clippy -p fpas-vm --all-targets -- -D warnings` passed after the fix.
- `cargo clippy --workspace --all-targets -- -D warnings` passed after the fix.
