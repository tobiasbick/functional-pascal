# fpas-fmt Review

## Summary

The formatter crate passes package Clippy. Emitter modules were split by AST sub-responsibility; no `fpas-fmt` source file now exceeds 400 lines.

## Findings

### Resolved: Emitter files were large enough to slow behavior-specific review

Evidence (before split): `crates/fpas-fmt/src/emit/decl.rs` had 467 lines, `stmt.rs` had 451 lines, and `expr.rs` had 428 lines.

Remediation (2026-07-04):

- `emit/decl/` — `group.rs` (declaration block grouping), `item.rs` (individual declarations), `mod.rs` (public API + tests).
- `emit/stmt/` — `loops.rs` (control flow), `line.rs` (layout helpers and `var` statements), `mod.rs` (dispatcher + tests).
- `emit/expr/` — `binary.rs` (operators and line-breaking), `literal.rs` (aggregates and formatting helpers), `mod.rs` (primary/postfix emission + tests).

Largest file after split: `emit/expr/mod.rs` (~264 lines).

## Verification

- `cargo test -p fpas-fmt` passed (unit, golden, round-trip, fuzz).
- `cargo clippy -p fpas-fmt --all-targets -- -D warnings` passed.
