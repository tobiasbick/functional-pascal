# AP19.2: Comparisons on distinct types

Package: [AP19: Distinct domain types](README.md)

## Scope

Inherit equality and ordering from the underlying type for operands of the
same distinct type.

## Prerequisites

- AP19.1 (distinct types).
- The open decision on constraints, dictionary keys, and `case` labels.

## Implementation

- Sema: allow `=`, `<>`, `<`, `>`, `<=`, `>=` when both operands have the same
  distinct type and the underlying type supports the operator; reject mixed
  domains and distinct-versus-underlying comparisons with a conversion hint.
- Apply the recorded decision on constraints, dictionary keys, and `case`
  labels (see the open decision in the [package README](README.md)).

## Affected areas

- `crates/fpas-sema/src/check/expr/operators.rs`, `check/expr/equality.rs`,
  constraint checking; compiler comparison lowering.

## Migration

None.

## Documentation

- The distinct-type documentation from AP19.1.

## Verification

- Same-type equality and ordering, mixed-domain rejection, comparison with the
  underlying type rejected, and the decided behavior for constraints,
  dictionary keys, and `case` labels.
