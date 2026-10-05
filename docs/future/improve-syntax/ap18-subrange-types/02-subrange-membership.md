# AP18.2: Subrange membership

Package: [AP18: Subrange types](README.md)

## Scope

Add `X in Percent`, which checks an integer value against a subrange before
conversion.

## Prerequisites

- AP18.1 (subrange types).
- AP07.3 (precedence of `in`).

## Implementation

- Sema: `in` with a subrange type on the right accepts integer and subrange
  values and yields `boolean`.
- Compiler: range check without conversion.

## Affected areas

- `crates/fpas-sema/src/check/expr/operators.rs`, compiler expression lowering.

## Migration

None.

## Documentation

- The subrange page and `operators.md`.

## Verification

- Membership at both endpoints and just outside; use in conditions with `and`;
  rejection of non-integer operands.
