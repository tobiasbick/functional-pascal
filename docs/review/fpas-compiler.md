# fpas-compiler Review

## Summary

The compiler crate has a good thematic directory structure, but package Clippy could not complete because `fpas-vm` fails as a dependency under `-D warnings`.

## Findings

### Medium: Compiler Clippy is blocked by VM dependency warnings

Evidence: `cargo clippy -p fpas-compiler --all-targets -- -D warnings` failed while checking `fpas-vm`, specifically `crates/fpas-vm/src/vm/shared.rs:51`.

Impact: Compiler-only changes cannot currently use strict package Clippy as a clean verification signal. That weakens review isolation because compiler regressions can be hidden behind VM lint failures.

Suggested fix: Fix the `fpas-vm` Clippy blockers first, then rerun `cargo clippy -p fpas-compiler --all-targets -- -D warnings`.

### Low: Large compiler test files are over the repository structure threshold

Evidence: `crates/fpas-compiler/src/tests/case_of/negative.rs` has 622 lines, `crates/fpas-compiler/src/tests/std_library/graph.rs` has 484 lines, `crates/fpas-compiler/src/tests/functions.rs` has 445 lines, and `crates/fpas-compiler/src/tests/basics.rs` has 421 lines.

Impact: These are test files, so runtime risk is low, but the files now mix many behavior themes. Future compiler changes will be harder to review surgically.

Suggested fix: Split by behavior under existing themed directories, for example scalar-case diagnostics, enum-case diagnostics, graph drawing calls, graph event behavior, function declarations, nested functions, and mutable parameters.

## Verification

- `cargo clippy -p fpas-compiler --all-targets -- -D warnings` failed because `fpas-vm` fails.

