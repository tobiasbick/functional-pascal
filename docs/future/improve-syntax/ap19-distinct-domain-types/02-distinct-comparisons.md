# AP19.2: Comparisons on distinct types

Package: [AP19: Distinct domain types](README.md)

Status: complete.

## Result

Two operands of the same distinct type compare with `=`, `<>`, `<`, `>`, `<=`,
and `>=`, using the underlying type's ordering. Mixed domains and
distinct-versus-underlying comparisons report both types and name the explicit
conversion. Records and other aggregates with distinct fields compare
structurally.

`in` follows equality for arrays and dictionary keys of the same distinct type;
substring `in` and logical operators stay unavailable. A distinct type may be a
dictionary key and satisfies `Comparable`, but not `Numeric` or `Printable`.

A distinct value over `integer`, `string`, or `boolean` is a `case` selector.
Labels and range endpoints are compile-time constants of the same distinct type,
written as a conversion (`UserId(2)`) or a `const`. A `Name(...)` label followed
by `..` parses as a value range. A single `Name(Value)` label keeps its pattern
form; Sema treats it as a value when `Name` is a distinct type, at the top level
and nested in patterns such as `Some(UserId(1))` or `is` tests. A
`distinct real` selector has its own diagnostic.

The compiler uses the underlying type for every operation on a distinct value,
so comparisons, ranges, and membership lower like their scalar counterparts.
The debugger evaluates comparisons on runtime values without additional
metadata.

## Ownership

- `crates/fpas-sema/src/check/expr/distinct_operators.rs` checks comparisons,
  membership, and the remaining rejected operators;
  `check/expr/equality.rs` and `types/mod.rs` (`Comparable`) admit distinct
  values.
- `crates/fpas-sema/src/check/stmt/control_flow/if_case/distinct_labels.rs`
  checks conversion labels and selector comparison types; `labels.rs`,
  `patterns/mod.rs`, and `patterns/values.rs` dispatch to it.
- `fpas_parser::Pattern::conversion_argument` identifies `Name(Value)` labels
  for Sema and the compiler; `parser/patterns.rs` detects call ranges.
- `crates/fpas-compiler/src/lowering/context/expressions.rs` erases distinct
  expression types; `lowering/case/{scalar.rs,patterns.rs}` lower conversion
  labels.

## Regression coverage

- Sema tests for same-type comparisons, records and options, mixed operands,
  membership, logical operators, and constraints; `case` tests for constants,
  conversions, ranges, guards, nested and `is` patterns, mismatched and
  non-constant labels, and `distinct real` selectors.
- A parser test for call ranges and single conversion labels.
- `tests/runner/distinct_comparisons_test.fpas` runs comparisons, `Comparable`,
  membership, dictionary keys, and `case` end to end.

## Current documentation

- [Distinct types](../../../pascal/language/types/distinct-types.md)
  (comparisons and `case`).
- [Generics](../../../pascal/language/types/generics.md) constraint table,
  [case intro](../../../pascal/language/control-flow/case-of-intro.md), and
  [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_label`).
