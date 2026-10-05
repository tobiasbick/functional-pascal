# AP18.1: Subrange declarations and conversions

Package: [AP18: Subrange types](README.md)

## Scope

Add integer subrange declarations `type Percent = 0..100;` with checked
conversion `Percent(X)`, implicit widening to `integer`, and integer
arithmetic results.

## Prerequisites

- AP16.1 (compile-time constant classification).

## Implementation

- Parser: subrange type bodies with static integer bounds; reject non-integer
  bounds and explicit base-type annotations.
- Sema: subrange types; implicit widening to `integer`; arithmetic yields
  `integer`; implicit narrowing in assignments, arguments, and returns is an
  error that shows the checked conversion.
- Conversion: compile-time error for out-of-range constants; runtime check
  with a panic reporting value and range for dynamic values.
- Unit interfaces export subrange types.

## Affected areas

- `crates/fpas-parser/src/parser/decl/data/type_defs.rs`, `decl/type_expr.rs`.
- `crates/fpas-sema/src/check/decl/types/`, `check/expr/operators.rs`,
  conversion checking.
- `crates/fpas-compiler/src/lowering/types/`, `fpas-vm` range check and panic
  diagnostic; `fpas-unit` interfaces.

## Migration

None.

## Documentation

- New subrange page under `docs/pascal/language/types/`, type index,
  `docs/specs/grammar.ebnf`, panic documentation.

## Verification

- Both endpoints and values just outside; invalid constants; dynamic
  conversion and panic details; widening; arithmetic result types; rejected
  implicit narrowing in assignments, arguments, and returns; rejected
  non-integer bounds and base types; imported subranges.
- If AP22.1 is merged: local inference from the explicit conversion.
