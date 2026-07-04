# fpas-fmt Review

## Summary

The formatter crate passed package Clippy. The main concern is structural size in the emitter modules.

## Findings

### Low: Emitter files are large enough to slow behavior-specific review

Evidence: `crates/fpas-fmt/src/emit/decl.rs` has 467 lines, `crates/fpas-fmt/src/emit/stmt.rs` has 451 lines, and `crates/fpas-fmt/src/emit/expr.rs` has 428 lines.

Impact: Formatter changes are often subtle and snapshot-sensitive. Large emitter files make it harder to isolate formatting behavior by source construct.

Suggested fix: Split emitters by AST sub-responsibility when the next formatter change touches them. Candidate splits: declaration grouping versus individual declarations, case/loop statements versus simple statements, and primary/postfix expressions versus binary expressions.

## Verification

- `cargo clippy -p fpas-fmt --all-targets -- -D warnings` passed.

