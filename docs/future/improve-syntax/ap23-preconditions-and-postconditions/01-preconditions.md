# AP23.1: Preconditions

Package: [AP23: Preconditions and postconditions](README.md)

## Scope

Add `requires` clauses on routine declarations, checked on every entry in all
build modes. Reserve both `requires` and `ensures`.

## Prerequisites

- AP07.3 (logical precedence), AP13.3 (routine closers), AP16.1 (constant
  classification).

## Implementation

- Lexer: reserve `requires` and `ensures`; diagnose their use as identifiers.
- Parser: `requires Expression;` clauses after the routine heading, for named
  routines and methods.
- Sema: boolean typing; repeated clauses combine with `and`; allowed
  expressions are parameters, constants, operators, and built-in
  length/membership checks; other calls are rejected.
- Compiler and VM: check on entry, including release builds and indirect
  calls through function values; a violation panics with routine name, clause
  text, and parameter values.
- Unit interfaces carry the clauses so imported routines are checked in the
  callee as compiled.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, `crates/fpas-parser/src/parser/decl/routines.rs`,
  `crates/fpas-sema/src/check/decl/routines.rs`, compiler routine prologue,
  `fpas-vm` panic diagnostics, editor highlighting.

## Migration

Rename identifiers spelled `requires` or `ensures`.

## Documentation

- New contracts page under `docs/pascal/language/functions/`,
  `docs/pascal/language/error-handling/panic.md`, keyword list,
  `docs/specs/grammar.ebnf`.

## Verification

- Entry checks, repeated clauses, boolean typing, rejected calls and `old`,
  release-build checks, panic details, indirect and imported calls, keywords
  as identifiers; violations do not become `Result` errors.
