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
  constant cannot turn a binding into a comparison; check typing, guards,
  duplicate arms, reachability, and exhaustiveness for nested patterns while
  keeping explicit top-level variant handling.
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
