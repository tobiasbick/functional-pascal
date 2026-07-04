# fpas-diagnostics Review

## Summary

No high-confidence correctness issue was found. The crate is intentionally small and passed package Clippy.

## Findings

### Low: Public diagnostic API is not lint-enforced for documentation coverage

Evidence: `crates/fpas-diagnostics/src/lib.rs:8` publicly exposes `codes`, and `crates/fpas-diagnostics/src/lib.rs:10` to `crates/fpas-diagnostics/src/lib.rs:12` re-export the public data model. The workspace lints in `Cargo.toml` do not include `missing_docs` or `rustdoc::broken_intra_doc_links`.

Impact: Diagnostics are user-facing and cross-crate. Missing or stale rustdoc on diagnostic types can make it harder to keep compiler, parser, runtime, and CLI error contracts aligned.

Suggested fix: Add crate-level documentation coverage incrementally, then consider `#![warn(missing_docs)]` and `#![warn(rustdoc::broken_intra_doc_links)]` for this crate first because it is compact.

## Verification

- `cargo clippy -p fpas-diagnostics --all-targets -- -D warnings` passed.

