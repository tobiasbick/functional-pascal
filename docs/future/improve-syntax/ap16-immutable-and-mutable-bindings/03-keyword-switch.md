# AP16.3: Keyword switch to const and var

Package: [AP16: Immutable and mutable bindings](README.md)

## Scope

Make `var` the reassignable binding and remove the `mutable` keyword from
bindings and parameters. This is the single keyword switch shared with AP17:
`mutable` parameters become read-only parameters with a local `var` copy where
the routine reassigned them.

## Prerequisites

- AP16.2 (no immutable `var` remains).

## Implementation

- Parser and sema: `var` declares a reassignable binding; `mutable var` and
  `mutable` parameters are rejected with a diagnostic showing `var` and the
  local-copy form respectively; assignment to a `const` suggests `var`.
- `for` loop variables are immutable per iteration without a keyword;
  assignment to them is an error. Verify current behavior first.
- Closures: a captured `var` shares one mutable cell, as `mutable var` did.
- Caller-mutating intrinsics such as `Push` and `Pop` keep their current
  special rule, which now requires a `var` binding instead of a `mutable var`;
  AP17.3 replaces that rule.
- Lexer: remove `mutable`; it becomes an identifier.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, parser declarations and
  parameters, `crates/fpas-sema/src/check/decl/vars.rs`, routine parameter
  checking, closure capture, editor highlighting and snippets.

## Migration

- Rewrite `mutable var` to `var` everywhere.
- For each `mutable` parameter, remove the modifier and, where the routine
  reassigns it, introduce a local `var` copy at the start of the body. Never
  turn local reassignment into caller mutation.
- All repository consumers, including generated declarations, fixtures, and
  documentation examples.

## Documentation

- `docs/pascal/language/basics/variables.md`, `local-variables.md`,
  `docs/pascal/language/functions/mutable-parameters.md` (remove, or replace by
  the read-only parameter rule on `parameters.md`), `closures.md`, keyword
  list, `docs/specs/grammar.ebnf`, authoring skill.

## Verification

- Reassignment of `var`, rejection for `const`, loop variables, captures of
  `const` and `var`, shared values, removed `mutable` with hints.
- Each migrated binding and parameter keeps its mutability and behavior.
- Full FPAS suite and example/app checks.
