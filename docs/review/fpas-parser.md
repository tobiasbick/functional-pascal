# fpas-parser Review

## Summary

No high-confidence parse correctness issue was found in this pass. The crate passed package Clippy.

## Findings

### Low: Public parse diagnostic enum lacks item-level docs

Evidence: `crates/fpas-parser/src/lib.rs:41` defines public `ParseDiagnostic` without a doc comment.

Impact: Parser callers receive mixed lexer/parser diagnostics through this enum. Without rustdoc, callers must inspect source to learn when `Lexer` versus `Parser` is produced and when `as_parser_error` intentionally filters out lexer diagnostics.

Suggested fix: Add `///` docs to `ParseDiagnostic` and its variants, and document that `as_diagnostic` normalizes both variants to the shared diagnostic type.

## Verification

- `cargo clippy -p fpas-parser --all-targets -- -D warnings` passed.

