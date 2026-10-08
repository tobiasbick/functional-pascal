# AP07.1: Short-circuit evaluation

Package: [AP07: Boolean rules](README.md)

Status: complete.

## Result

Boolean `and` and `or` evaluate left to right and skip an unnecessary right
operand. `xor` evaluates both operands. A skipped operand performs no calls,
writes, name resolution at execution, `try` propagation or debugger-budget work;
parsing and static validation still apply to the whole expression.

Branch-based lowering lives in
`crates/fpas-compiler/src/lowering/expr/boolean.rs`. Constant folding preserves
guarded effects. The VM executes the branches; debugger expression evaluation
uses the same short-circuit rule.

## Regression coverage

`tests/runner/boolean/`, compiler control-flow tests, VM debugger tests and
debugger protocol tests cover traces, skipped failures, nested expressions,
`try`, and budgets. See [operators](../../../pascal/language/basics/operators.md).
