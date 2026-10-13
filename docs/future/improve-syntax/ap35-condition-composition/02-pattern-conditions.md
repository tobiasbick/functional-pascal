# AP35.2: Pattern conditions

Package: [AP35](README.md). Status: complete.

## Implemented behavior

`Value is Pattern` is a nonbinding Boolean expression in ordinary value and
condition positions, including grouping, logical operators, guards, while/until,
arguments, returns, and restricted contracts. Every `const Name` inside its
pattern is rejected (FP3034), including nested bindings. Whole-value `_` is
rejected with or without grouping; payload wildcards remain valid.

Only case arms introduce pattern names. Existing case scope, coverage, pattern
constant rules, comparison precedence, Boolean grouping, short-circuit behavior,
and contract operand/native-operation restrictions remain. This replaces
AP20.3's original binding and condition-only semantics.

## Implementation owners

- Sema: `check/expr/is_test.rs` and shared `if_case/patterns/` validate predicates
  without declaring names. Ordinary branch conditions use the expression checker.
- Compiler: `lowering/expr/is_test.rs` evaluates the selector once and reuses
  `lowering/case/patterns.rs`; matching and failing paths produce a Boolean.
  Ordinary expression/branch lowering owns logical and loop evaluation.
- Parser and formatter retain comparison precedence and shared pattern syntax.
- Closure capture, task-escape analysis, and language-service symbol extraction
  traverse `is` without introducing names. Case bindings retain their own scopes.
- The debugger retains its documented restricted read-only expression subset;
  this package does not extend that evaluator to pattern or case expressions.

## Migration and documentation

Library, app, example, FPAS/Rust fixture, formatter, skill, and handbook consumers
extract payloads with exhaustive case statements or expressions. False guards
retain their fallback, selectors are not evaluated again to retrieve payloads,
and loop break/continue targets and closure captures remain intact.

Current rules: [is tests](../../../pascal/language/pattern-matching/is-test.md),
pattern/control-flow pages, compact reference, authoring skill, diagnostics,
formal grammar, and AP20/AP03 records. AP39 owns coverage changes; AP43 owns
broader absence-handling guidance.

## Regression coverage

- `fpas-sema/src/tests/stmt/is_tests.rs`: accepted Boolean contexts, rejected
  bindings at any depth, pattern validation, and transparent wildcard rejection.
- Parser is tests and formatter case regressions: precedence and round trips.
- Compiler `tests/control_flow/pattern_bindings.rs`: single evaluation, skipped
  logical operands, nested comparisons, loop reevaluation, guarded migration
  effects, and comparison captures.
- Language-service `tests/semantic_tools/decision_bindings.rs`: rejected names
  have no definitions, rename targets, or completions; case identities remain.
- `tests/runner/is_pattern_test.fpas`, pattern-resolution/capability fixtures,
  source-library suites, app/example projects, and checked handbook programs.

Repository checks and an independent review cover the complete delivery.
