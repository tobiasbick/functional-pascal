# fpas-project Review

## Summary

The crate passed package Clippy. It has a clear thematic structure for loading, linking, workspace handling, paths, and test manifests.

## Findings

### Low: Public project APIs return plain `String` errors

Evidence: `crates/fpas-project/src/loading/mod.rs:17`, `crates/fpas-project/src/link/mod.rs:41`, and `crates/fpas-project/src/link/mod.rs:57` expose public `Result<_, String>` APIs.

Impact: CLI callers can print these messages, but other callers cannot reliably inspect error kind, path, or source stage without string matching. This also differs from the diagnostic model used by lexer/parser/sema/runtime crates.

Suggested fix: Introduce a crate-level project error enum when the next project-loading feature lands. Keep display strings LLM-readable, but expose structured variants for manifest parse, path resolution, duplicate units, export validation, and link failures.

## Verification

- `cargo clippy -p fpas-project --all-targets -- -D warnings` passed.

