# fpas-bytecode Review

## Summary

No high-confidence correctness issue was found in the reviewed slice. The crate is small, focused, and passed package Clippy.

## Findings

### Low: Crate root has no module-level API documentation

Evidence: `crates/fpas-bytecode/src/lib.rs:1` starts directly with module declarations and public re-exports.

Impact: The crate exposes core VM contract types (`Chunk`, `Op`, `Value`, and intrinsic enums), but generated rustdoc does not explain the boundary between bytecode format, VM execution, and standard-library intrinsic numbering. This makes public API drift harder to review.

Suggested fix: Add a `//!` crate-level doc comment describing bytecode ownership, intrinsic stability expectations, and the intended consumers (`fpas-compiler`, `fpas-vm`, and `fpas-std`). Consider enabling doc linting later once existing public docs are filled in.

## Verification

- `cargo clippy -p fpas-bytecode --all-targets -- -D warnings` passed.

