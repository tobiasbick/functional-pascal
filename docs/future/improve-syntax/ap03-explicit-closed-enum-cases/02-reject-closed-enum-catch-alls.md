# AP03.2: Reject catch-alls on closed enums

Package: [AP03: Explicit closed-enum cases](README.md)

Status: complete.

## Result

FP3035 rejects `else` in a `case` over a user enum, `Option`, or `Result`,
including aliases and imported types. With valid labels, the diagnostic names
one missing pattern per uncovered variant, including nested payloads. Its hint
recommends explicit `when` arms, `null;` for no action, or an `is` test when only
one variant matters. Complete explicit coverage still rejects `else`, with a
hint to remove the redundant branch.

Cases without `else` report FP3011 for missing patterns. Guarded arms do not
complete coverage. `_` ignores a payload position and cannot cover another
variant. Invalid labels do not produce misleading redundant-branch hints;
rejected bodies are checked without leaking their bindings.

Scalar `integer`, `string`, and `boolean` cases retain `else`. Boolean cases
do not require both `true` and `false`.

## Ownership

- `crates/fpas-sema/src/check/stmt/control_flow/if_case/exhaustiveness.rs`
  owns catch-all and missing-pattern diagnostics, called from `mod.rs`.
- The existing `if_case/coverage.rs` pattern matrix supplies coverage and
  missing-pattern witnesses; it is unchanged.
- `if_case/scalar_bindings.rs` limits replacement hints to valid forms.
- `crates/fpas-diagnostics/src/codes.rs` allocates FP3035.
- Sema and CLI `closed_enum_cases.rs` modules own the regression tests.

## Regression coverage

Tests cover simple and data enums, `Option`, `Result`, aliases, redundant
catch-alls, guards, nested patterns, payload wildcards, scalar exceptions,
diagnostic recovery, and CLI text and JSON output. Adding an enum variant
reports every incomplete case while accepting the extended complete case.
An imported-enum test verifies the same behavior with an import alias and an
existing source-adjacent `.fpascu` sidecar.

Verification includes Rust formatting, build and workspace tests, FPAS
formatting and the complete FPAS suite, affected app/example checks, and
documentation links.

## Current documentation

- [Exhaustiveness](../../../pascal/language/pattern-matching/exhaustiveness.md),
  [`case`](../../../pascal/language/control-flow/case-of-intro.md),
  [pattern syntax](../../../pascal/language/pattern-matching/syntax.md), and
  [scalar labels](../../../pascal/language/pattern-matching/scalar-labels.md).
- [Diagnostics](../../../pascal/tools/diagnostics.md),
  [grammar](../../../specs/grammar.ebnf), and the FPAS authoring skill.
