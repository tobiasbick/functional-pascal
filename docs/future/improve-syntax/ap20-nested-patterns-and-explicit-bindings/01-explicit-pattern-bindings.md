# AP20.1: Explicit pattern bindings

Package: [AP20: Nested patterns and explicit bindings](README.md)

## Scope

Replace plain-identifier pattern bindings with `const Name` and add `_` for an
ignored field. This is the binding rule shared by flat and nested patterns.

## Prerequisites

- AP13.5 (`when` case arms), so patterns are migrated once in the final arm
  syntax.
- AP16.1 (scalar label and guard-binding classification).
- The scalar guard-binding rule in the [package README](README.md#decisions)
  (agreed).

## Implementation

- Parser: `const Name` and `_` in enum, `Option`, and `Result` patterns, and
  `when const N if Guard:` for scalar guard bindings. A plain identifier in a
  payload position compares with a compile-time constant; an unknown name
  gets a diagnostic showing `const Name`.
- Sema: a bare scalar label is always a value comparison with a compile-time
  constant or enum member; an unknown name reports the `const N` replacement.
  `when const N:` requires a guard and the arm's only label.
- Sema: bindings are scoped to the arm; a duplicate binding name in one
  pattern is an error.

## Affected areas

- `crates/fpas-parser/` case labels and patterns (`destructure_pattern`,
  `enum_pattern`), `crates/fpas-sema/src/check/stmt/control_flow/if_case/`
  (bindings), `fpas-fmt`.

## Migration

Rewrite every pattern binding to `const Name` in all repository consumers,
including the scalar guard bindings in
`examples/pascal/pattern-matching/guards.fpas` and
`tests/runner/immutable_binding_migration_test.fpas`.

## Documentation

- `docs/pascal/language/pattern-matching/` (syntax, enum and Result/Option
  patterns, scalar guard bindings in `guards.md`), `docs/specs/grammar.ebnf`.

## Verification

- Bindings in every pattern kind, `_`, duplicate names, arm scope, plain
  identifier diagnostic; scalar `const N` guard bindings, bare scalar labels
  that name constants, enum members, and unknown names, and `const N` without
  a guard; FPAS suite after migration.

## Result

- Parser: case labels use `CaseLabel::Binding` for scalar `const N` and
  `CaseLabel::Pattern(Pattern)` for recursive variant and destructuring
  patterns. `Pattern::Binding`, `Pattern::Wildcard`, and `Pattern::Value`
  distinguish explicit bindings, ignored fields, and comparisons
  (`crates/fpas-parser/src/ast/patterns.rs`, `parser/patterns.rs`).
- Sema (`check/stmt/control_flow/if_case/patterns/`): unknown payload names
  and unknown bare scalar labels report FP3031 with the `const Name` form.
  Named compile-time constants compare; literals and nested patterns are
  supported by AP20.2. Misplaced scalar `const N` (no guard, other labels,
  non-scalar case) reports FP3032. A call written as a scalar label reports
  FP3014. Scalar guard bindings are checked in `scalar_bindings.rs`.
- Compiler, formatter, closure capture, and source-id remapping use the
  recursive labels. Closure capture resolves compared values before entering
  the arm's binding scope, including when several labels share bindings.
- Migration: every repository pattern binding (`.fpas`, embedded Rust and
  TypeScript fixtures, `docs/pascal/`, grammar, authoring skill) now uses
  `const Name`; the two scalar guard bindings use `when const N if ...`.

Coverage: `crates/fpas-sema/src/tests/stmt/pattern_bindings.rs`,
`crates/fpas-parser/src/tests/stmt/conditionals/case_stmt.rs`,
`crates/fpas-fmt/tests/case_block_regressions.rs`,
`crates/fpas-compiler/src/tests/control_flow/pattern_bindings.rs`,
`tests/runner/pattern_bindings_test.fpas`,
`tests/runner/pattern_resolution_test.fpas`, and
`editors/vscode/scripts/verify-grammar.mjs`.
