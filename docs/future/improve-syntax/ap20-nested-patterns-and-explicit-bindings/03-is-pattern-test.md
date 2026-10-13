# AP20.3: Pattern test with is

Package: [AP20: Nested patterns and explicit bindings](README.md)

Status: complete. Its original condition-only binding rules are replaced by
[AP35.2](../ap35-condition-composition/02-pattern-conditions.md).

## Current implementation

- Lexer: `is` is reserved; identifier use reports the reserved-keyword hint.
- Parser: `Expr::Is` has comparison precedence and uses the shared recursive
  pattern parser. Compared pattern values remain additive expressions.
- Sema: `check/expr/is_test.rs` checks an ordinary Boolean expression.
  Shared `if_case/patterns/` validation rejects every explicit binding (FP3034)
  without creating scope names. Case bindings and coverage remain unchanged.
- Compiler: `lowering/expr/is_test.rs` evaluates the selector once, reuses
  `lowering/case/patterns.rs`, and joins matching/failing paths into a Boolean.
  Ordinary expression lowering preserves short-circuit and loop evaluation.
- Formatter, capture collection, source remapping, and editor scopes traverse
  the expression normally. Only case arms introduce pattern symbols.
- The debugger retains its documented restricted read-only evaluator;
  it rejects pattern tests and case expressions independently of source syntax.

## Documentation and coverage

Current behavior: [is tests](../../../pascal/language/pattern-matching/is-test.md),
control-flow pages, compact reference, authoring skill, and formal grammar.

Primary coverage: parser `tests/expr/is_tests.rs`, Sema `tests/stmt/is_tests.rs`,
compiler `tests/control_flow/pattern_bindings.rs`, formatter
`tests/case_block_regressions.rs`, language-service
`tests/semantic_tools/decision_bindings.rs`, and `tests/runner/is_pattern_test.fpas`.
