# AP18.1: Subrange declarations and conversions

Package: [AP18: Subrange types](README.md)

Status: complete.

## Scope

Integer subrange declarations `type Percent = 0..100;` with checked
conversion `Percent(X)`, implicit widening to `integer`, and integer
arithmetic results (`/` produces `real`) are implemented. The package's structural
range identity, exact `var` compatibility, generic constraints, and subrange case
rules apply. Membership is covered by [AP18.2](02-subrange-membership.md).

## Prerequisites

- AP16.1 (compile-time constant classification).
- AP22.1 (local inference for explicit conversions).
- The type-compatibility, operator, generic, and case decisions in the package
  README are agreed.

## Implementation

- Parser: subrange type bodies with expression bounds; Sema requires static
  integers. Explicit base-type annotations are rejected.
- Sema: equal evaluated bounds define the same type; implicit scalar widening to
  `integer`; integer arithmetic yields `integer`, `/` yields `real`; implicit
  narrowing in assignments, arguments, and returns is an
  error that shows the checked conversion.
- Conversion: compile-time error for out-of-range constants; runtime check
  with a panic reporting value and range for dynamic values.
- Unit interfaces export subrange types.
- `var` parameters and container/callable components retain exact types.
- Subranges satisfy `Comparable` and `Printable`, reject `Numeric`, and retain
  [AP39's finite coverage rules](../ap39-consistent-case-coverage/README.md)
  without `else`, in statements and expressions.

## Affected areas

- `crates/fpas-parser/src/parser/decl/data/subranges.rs` and the declaration AST.
- `crates/fpas-sema/src/check/decl/types/subranges.rs`,
  `check/expr/subrange_conversion.rs`, scalar operators and indexing,
  `types/subranges.rs`, `types/compatibility.rs`, and case conversion labels.
- `crates/fpas-compiler/src/lowering/calls/subranges.rs` emits range checks and
  uses the existing VM panic path; scalar layouts use integer representation.
- `crates/fpas-unit/src/interface/types.rs` and Sema interface conversion/export.
- Formatter, project source maps, and language-service type navigation.

Type compatibility is split out of `types/mod.rs`; case conversion labels share
`check/stmt/control_flow/if_case/conversion_labels.rs`. Static-bound AST inputs
are retained with stable expression identity through semantic analysis.

## Migration

None.

## Documentation

- [Subrange types](../../../pascal/language/types/subranges.md), type index,
  declaration order, operators, generics, local inference, case exhaustiveness,
  panic documentation, and `docs/specs/grammar.ebnf`.

## Verification

- Both endpoints and values just outside; invalid constants; dynamic
  conversion and panic details; widening; arithmetic result types; rejected
  implicit narrowing in assignments, arguments, and returns; rejected
  non-integer bounds and base types; imported subranges.
- Local inference from the explicit conversion, including imported types.
- Equal and different bounds, aliases and cross-unit compatibility; exact `var`
  parameters; invariant container/callable types; `/` result type; generic
  constraints; scalar case labels and explicit finite coverage in both positions.
- Singleton and full-width integer bounds; static record fields and conditional
  expressions; declaration-order visibility; arity and positional-only calls;
  scalar indices and container membership; evaluation once, skipped branches,
  captures, and nested conversion labels.
- Real CLI diagnostics and imported aliases through fresh, reused, and rebuilt
  units; formatter comment preservation; inferred hover and type navigation;
  `tests/runner/subrange_test.fpas` through the ordinary test runner.
