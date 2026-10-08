# AP20.2: Nested patterns

Package: [AP20: Nested patterns and explicit bindings](README.md)

## Scope

Allow patterns inside patterns, with literals and named constants as
comparisons, for example `when Ok(Some(const User)):`.

## Prerequisites

- AP20.1 (explicit bindings).

## Implementation

- Parser: recursive patterns in payload positions.
- Sema: resolve constants and constructors explicitly so that a nearby
  constant cannot turn a binding into a comparison. A bare identifier in a
  payload position must be a compile-time constant; other names report the
  `const Name` replacement. `_` on an enum-typed field covers all its variants.
  Check typing, guards, duplicate arms, reachability, and recursive
  exhaustiveness for nested patterns while keeping explicit top-level variant
  handling; guards do not count toward coverage.
- Compiler: nested matching in case lowering.

## Affected areas

- Parser patterns; `crates/fpas-sema/src/check/stmt/control_flow/if_case/`
  (labels, exhaustiveness); `crates/fpas-compiler/src/lowering/case/`.

## Migration

None; existing flat patterns remain valid.

## Documentation

- `docs/pascal/language/pattern-matching/` (syntax, exhaustiveness, guards).

## Verification

- Nested `Option`/`Result`/enum patterns, literal and constant comparisons,
  `_` in nested positions, shadowing, constructor lookup, unreachable and
  duplicate arms, missing nested cases, guards.

## Result

- AST: `CaseLabel::Pattern(Pattern)` replaces the flat variant and destructure
  labels; `Pattern` is recursive (`Binding`, `Wildcard`, `Value`, `Variant`,
  `Destructure`). The parser reads patterns recursively under the nesting
  limit.
- Sema (`check/stmt/control_flow/if_case/`): `patterns/mod.rs` types nested
  patterns, binds names once per label, and accepts comparisons with literals,
  compile-time constants, and enum members of ordinal, enum, and string types
  (other types: SEMA_TYPE_MISMATCH; computed constants: FP3014; unknown
  names: FP3031). `coverage.rs` implements a pattern-matrix check:
  exhaustiveness names one missing pattern per uncovered variant (for example
  `Ok(None)`), and labels already covered by earlier unguarded arms report
  FP3033. `patterns/values.rs` uses the existing scalar constant evaluator to
  normalize comparisons: named boolean and simple-enum constants count as
  their finite values, and equal integer or string comparisons are duplicates.
  Comparisons of open types never complete coverage. `patterns/constructors.rs`
  resolves the complete constructor name through ordinary lexical and qualified
  lookup and checks that it belongs to the matched enum. Unit and type aliases
  work; unknown or wrong qualifiers are rejected. Scalar guard bindings are
  checked in `scalar_bindings.rs`.
- Compiler (`lowering/case/patterns.rs`): recursive tests branch to the next
  label on a mismatch and keep matched payloads in hidden locals, so nested
  tests and bindings read them after earlier branches.
- Formatter, closure capture, and source-id remapping handle nested patterns.
  Comparison expressions resolve before any binding names from the arm are
  introduced, so a same-named binding cannot suppress a closure capture.

Coverage: `crates/fpas-sema/src/tests/stmt/nested_patterns.rs`,
`crates/fpas-compiler/src/tests/control_flow/pattern_bindings.rs`,
`crates/fpas-parser/src/tests/stmt/conditionals/case_stmt.rs`,
`crates/fpas-fmt/tests/case_block_regressions.rs`,
`tests/runner/nested_patterns_test.fpas`,
`tests/runner/pattern_resolution_test.fpas`,
`crates/fpas-build/tests/import_aliases.rs` (cold and reused unit artifacts),
and `crates/fpas-cli/src/main_tests/projects/enum_variants.rs` (diagnostics
through `check` and `run`).
