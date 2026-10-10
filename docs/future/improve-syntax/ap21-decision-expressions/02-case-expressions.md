# AP21.2: case expressions

Package: [AP21: Decision expressions](README.md)

Status: complete.

## Result

`case Value of when Labels [if Guard]: Expression; ... [else Expression;] end case`
is a primary expression. A statement that starts with `case` stays the `case`
statement. Each arm is one expression ended by `;`; a statement or `:=` inside an
arm reports FP2020, and a missing `;` or `end case` uses the existing diagnostics.

Labels, guards, scalar guard bindings, pattern bindings, arm scopes, distinct
labels, and unreachable-label checks are shared with the `case` statement. Every
input must produce a value: enum, `Result`, `Option`, and `boolean` selectors need
unguarded coverage of every value (FP3011, with expression wording), closed enums
reject `else` (FP3035), and `integer`, `string`, and distinct selectors need an
`else` arm. Arm values share one type under the AP21.1 rules. A case whose
selector, labels, and values are compile-time constants, without guards,
bindings, or destructuring patterns, is a constant and folds to the selected
value.

The compiler reuses the scalar and variant case lowering through an arm-body
callback and stores the selected value in a hidden local; an expression without
`else` panics if no arm matches. The formatter writes one arm per line and keeps
arm comments. Debugger evaluation rejects `case` expressions with a hint.

## Ownership

- `crates/fpas-parser/src/parser/expr/case_expression.rs`; `ast/expr.rs`
  (`Expr::Case`, `CaseExprArm`, `CaseExprElse`).
- `crates/fpas-sema/src/check/stmt/control_flow/if_case/arms.rs` (shared arm
  checks), `case_expression.rs` (coverage and arm types), and
  `exhaustiveness.rs` (`CaseForm` wording); `check/decl/consts/` classify and
  fold constants.
- `crates/fpas-compiler/src/lowering/case/` (`CaseArmHead`, `CaseBody`,
  `CaseEnding`, `expression.rs`).
- `crates/fpas-fmt/src/emit/expr/case_expression.rs` and the comment traversal.

## Regression coverage

- Parser tests for arms, guards, patterns, ranges, `else`, operand and `return`
  positions, statement-start handling, FP2020, missing `;`, and missing
  `end case`.
- Sema tests for enums, data enums, `Option`, booleans, scalar ranges, guard
  bindings, distinct labels, context typing, incomplete coverage, closed-enum
  `else`, arm type mismatches, expected types, and constants.
- Formatter golden and comment tests; a debugger rejection test.
- `tests/runner/case_expressions_test.fpas` runs data enums, simple enums,
  ranges, guards, `Option`, booleans, distinct labels, single evaluation, and a
  constant `case` label.

## Current documentation

- [Case expressions](../../../pascal/language/control-flow/case-of-intro.md#case-expressions),
  [exhaustiveness rules](../../../pascal/language/pattern-matching/exhaustiveness.md#rules),
  [formatter style](../../../pascal/tools/fmt-style.md),
  [debugger](../../../pascal/tools/debugger.md),
  [diagnostics](../../../pascal/tools/diagnostics.md) (FP2020), and
  [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_expression`).
