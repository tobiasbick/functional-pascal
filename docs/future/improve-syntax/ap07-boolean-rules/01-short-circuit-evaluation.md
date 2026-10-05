# AP07.1: Short-circuit evaluation

Package: [AP07: Boolean rules](README.md)

## Scope

Specify and implement left-to-right short-circuit evaluation for boolean `and`
and `or`. `xor` stays eager.

## Prerequisites

None.

## Implementation

- Verify the current behavior in compiler lowering, constant folding, the VM,
  and debugger expression evaluation.
- Lower boolean `and`/`or` with branches; move boolean lowering into a focused
  module instead of growing `crates/fpas-compiler/src/lowering/expr.rs`
  (490 lines at planning time).
- Debugger watch evaluation follows the same rule.
- Integer `and`/`or` keep their current eager behavior until AP07.4.

## Affected areas

- `crates/fpas-compiler/src/lowering/expr.rs` and a new boolean lowering module.
- `crates/fpas-compiler/src/optimize/` (constant folding).
- `crates/fpas-vm/src/vm/debug/` (expression evaluation).

## Migration

Check repository sources for right operands with side effects whose skipping
would change behavior; record each case in the pull request and adapt it.

## Documentation

- `docs/pascal/language/basics/operators.md`: evaluation order and
  short-circuiting.

## Verification

- Execution tests with side-effect traces: skipped right operand for
  `false and X` and `true or X`; evaluated for the other cases; eager `xor`;
  a failing right operand that is skipped; `try` inside a skipped operand.
- Debugger evaluation tests for the same cases.
