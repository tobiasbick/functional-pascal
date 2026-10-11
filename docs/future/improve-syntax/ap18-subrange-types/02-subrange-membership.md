# AP18.2: Subrange membership

Package: [AP18: Subrange types](README.md)

Status: complete.

## Scope

`X in Percent` checks an integer or subrange value against a subrange type's
inclusive bounds and returns `boolean` before explicit conversion.

## Prerequisites

- AP18.1 (subrange types).
- AP07.3 (precedence of `in`).

## Implementation

- Sema: `in` with a subrange type on the right accepts integer and subrange
  values and yields `boolean`. Aliases and imported qualified types preserve
  their bounds; value operands retain collection and string membership.
- Other type operands and non-integer inputs are rejected with syntax hints.
  A subrange value on the right does not name a range to test.
- Static membership is a compile-time constant; computed inputs retain their
  runtime classification.
- Compiler: evaluate the input once and compare both inclusive bounds without
  conversion or overflow. Out-of-range values return `false`.
- Boolean precedence and short-circuit rules apply; membership does not narrow
  the input's type or remove the need for checked conversion.

## Affected areas

- `crates/fpas-sema/src/check/expr/subrange_membership.rs`, expression metadata,
  scalar constant folding, and constant classification.
- `crates/fpas-compiler/src/lowering/expr/subranges.rs` owns the inclusive bound
  comparisons shared by membership and checked conversions.

## Migration

None.

## Documentation

- [Subrange membership](../../../pascal/language/types/subranges.md#membership)
  and [operators](../../../pascal/language/basics/operators.md).

## Verification

- `crates/fpas-compiler/src/tests/subranges.rs`: inclusive endpoints and values
  just outside, singleton and full-width integer bounds, subrange inputs,
  evaluation once, nested control flow, captures, skipped branches, and
  short-circuit guards before checked conversion.
- `crates/fpas-sema/src/tests/expr/subrange_membership.rs`: rejected
  non-integer inputs, type/value distinction, aliases, shadowing, static and
  computed classification, and rejected implicit narrowing after a guard.
- `crates/fpas-cli/tests/json_streams/subranges.rs`: imported membership through
  fresh, reused, and rebuilt units with changed bounds.
- `crates/fpas-language-service/tests/local_inference.rs`: Boolean hover and
  navigation to the membership type.
- `tests/runner/subrange_test.fpas`: membership and guarded conversion through
  the ordinary FPAS test runner.
