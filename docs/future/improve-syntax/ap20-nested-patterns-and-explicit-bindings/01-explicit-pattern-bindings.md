# AP20.1: Explicit pattern bindings

Package: [AP20: Nested patterns and explicit bindings](README.md)

## Scope

Replace plain-identifier pattern bindings with `const Name` and add `_` for an
ignored field, in the current flat patterns.

## Prerequisites

- AP13.5 (`when` case arms), so patterns are migrated once in the final arm
  syntax.

## Implementation

- Parser: `const Name` and `_` in enum, `Option`, and `Result` patterns. A
  plain identifier in a binding position gets a diagnostic showing
  `const Name`.
- Sema: bindings are scoped to the arm; a duplicate binding name in one
  pattern is an error.

## Affected areas

- `crates/fpas-parser/` case labels and patterns (`destructure_label`,
  `enum_pattern`), `crates/fpas-sema/src/check/stmt/control_flow/if_case/`
  (bindings), `fpas-fmt`.

## Migration

Rewrite every pattern binding to `const Name` in all repository consumers.

## Documentation

- `docs/pascal/language/pattern-matching/` (syntax, enum and Result/Option
  patterns), `docs/specs/grammar.ebnf`.

## Verification

- Bindings in every pattern kind, `_`, duplicate names, arm scope, plain
  identifier diagnostic; FPAS suite after migration.
