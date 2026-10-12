# AP33.2: Arithmetic and literals

Package: [AP33](README.md). Status: blocked by AP33.1.

## Scope

Implement the agreed numeric compatibility, arithmetic, and literal rules.
Keep compile-time evaluation and runtime execution consistent. Preserve
distinct/subrange identity and checked conversion boundaries.

## Affected areas

Inspect `crates/fpas-lexer/src/`,
`crates/fpas-sema/src/check/expr/operators.rs`, constant evaluation,
`crates/fpas-vm/src/vm/value_ops/integer.rs`, `scalar.rs`, `comparison.rs`,
compiler lowering, and numeric standard builtins. Enumerate concrete owners
before editing; do not expand Value layout or add numeric types as a side task.

## Migration and documentation

Migrate mixed arithmetic and boundary constants in all repository consumers
without changing results except where the decision explicitly requires it.
Update [operators](../../../pascal/language/basics/operators.md),
[literals](../../../pascal/language/basics/number-literals.md), conversion,
domain-type and generic references, grammar, and diagnostics together.

## Verification

Cover min/max values, signed minimum literals, checked or explicitly selected
wrapping behavior, division by zero, negative division/remainders, mixed-type
operations in every context, NaN/infinity/order, and constant/runtime parity.
Verify generic and compiled-unit paths; run applicable full implementation
checks from the development process.
