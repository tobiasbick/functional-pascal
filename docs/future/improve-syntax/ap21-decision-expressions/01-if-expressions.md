# AP21.1: if expressions

Package: [AP21: Decision expressions](README.md)

Status: complete.

## Result

`if C then A elsif D then B else E end if` is a primary expression. A statement
that starts with `if` stays the `if` statement; the expression form appears
after `:=`, `return`, or `discard`, as an argument, an operand, or a condition,
and needs no parentheses as an operand. Each branch is one expression; a missing
`else` (FP2019) and a statement or `;` inside a branch (FP2020) have dedicated
parser diagnostics, and a missing `end if` uses the named-ending diagnostic.

All branches have one type, checked like an assignment without numeric
widening; integer/real and distinct mismatches name the explicit conversion.
Context-typed branches such as `None` or `[]` take the type of the other
branches, and the shared type is checked against the expected type of the
position. Conditions are boolean and may use `is` tests whose bindings are
visible only in their branch value. An expression whose conditions and branches
are compile-time constants is itself a constant and folds to the selected value.

The compiler evaluates conditions in order and only the selected branch value,
storing it in a hidden local read after the merge. Closure discovery, capture,
task-bound, discard, and source-map traversals include branch values. The
debugger evaluates `if` expressions with the same selection rule; `is` tests
remain unsupported there.

The formatter keeps a fitting expression without comments on one line and
otherwise puts `elsif`, `else`, and `end if` on indented lines, preserving
comments at each branch and before the ending.

## Ownership

- `crates/fpas-parser/src/parser/expr/conditional.rs` parses the expression;
  `ast/expr.rs` defines `Expr::If` and `IfExprBranch`.
- `crates/fpas-sema/src/check/expr/decision.rs` checks branches and conditions;
  `check/decl/consts/{classification.rs,scalar.rs}` classify and fold
  constants.
- `crates/fpas-compiler/src/lowering/expr/decision.rs` lowers the expression.
- `crates/fpas-fmt/src/emit/expr/conditional.rs` and the comment traversal
  format it; `crates/fpas-vm/src/vm/debug/evaluation/` and
  `crates/fpas-debug/src/evaluation/validate.rs` evaluate it in the debugger.

## Regression coverage

- Parser tests for `elsif` chains, operand position, nesting, `is` conditions,
  statement-start handling, and FP2019/FP2020/missing `end if`.
- Sema tests for shared types, context typing, widening and distinct
  mismatches, expected types, boolean conditions, `is` binding scope, and
  constant classification.
- Formatter golden and comment tests; a debugger test for branch selection,
  short-circuiting, and non-boolean conditions.
- `tests/runner/if_expressions_test.fpas` runs values, `is` conditions, context
  typing, single evaluation, nesting, and a constant `case` label.

## Current documentation

- [If expressions](../../../pascal/language/control-flow/if-then-else.md#if-expressions),
  [operator precedence](../../../pascal/language/basics/operators.md#operator-precedence),
  [diagnostics](../../../pascal/tools/diagnostics.md) (FP2019, FP2020),
  [formatter style](../../../pascal/tools/fmt-style.md), and
  [`grammar.ebnf`](../../../specs/grammar.ebnf) (`if_expression`).
