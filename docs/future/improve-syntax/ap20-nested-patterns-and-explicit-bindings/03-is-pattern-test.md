# AP20.3: Pattern test with is

Package: [AP20: Nested patterns and explicit bindings](README.md)

## Scope

Add `Value is Pattern` as the condition of `if`, `elsif`, and `while`, with
bindings visible only in the branch or loop body.

## Prerequisites

- AP20.2 (pattern rules, including nesting).
- AP13.4 (`elsif`, statement-list bodies).
- AP07.3 (precedence of `is` and logical operators).

## Implementation

- Lexer: reserve `is`; diagnose its use as an identifier.
- Parser: `is` at comparison level with a pattern on the right.
- Sema: allowed only as an `if`/`elsif`/`while` condition, optionally followed
  by `and` conditions that may use the bindings; rejected under `or`, under
  `not`, and in any other position, with a diagnostic naming the allowed
  positions. No exhaustiveness check.
- Compiler: match and bind, then evaluate the following `and` conditions.
- Reuse the AP20.2 pattern parser (`parser/patterns.rs`), pattern
  checking (`check_is_pattern` in `if_case/patterns/mod.rs`), and recursive
  pattern tests (`lowering/case/patterns.rs`) instead of duplicating them.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, parser expressions,
  `crates/fpas-sema/src/check/stmt/control_flow/`, compiler control-flow
  lowering, editor highlighting.

## Migration

Rename identifiers spelled `is`. Single-variant `case` statements may move to
`is` tests in AP03.1.

## Documentation

- New section in `docs/pascal/language/pattern-matching/`, control-flow pages,
  keyword list, `docs/specs/grammar.ebnf`.

## Verification

- `is` in `if`, `elsif`, and `while`; binding scope; following `and`
  conditions using bindings; rejection under `or`, `not`, and as a value;
  nested patterns in `is`.

## Result

- Lexer: `is` is reserved (`Token::Is`); as an identifier it reports the
  reserved-keyword hint.
- Parser: `Expr::Is { value, pattern, span }` at comparison precedence, using
  the shared pattern parser in `crates/fpas-parser/src/parser/patterns.rs`.
  Compared values in patterns are additive expressions, so `X is 0 and Ready`
  stays a conjunction.
- Sema (`check/stmt/control_flow/conditions.rs`): `if`, `elsif`, and `while`
  split their top-level `and` chain; each `is` test checks its pattern with the
  case rules and defines its bindings for later conditions and the guarded
  body, which get their own scope. Any other position, including under `or`,
  `not`, parentheses, `case` guards, `repeat ... until`, and values, reports
  FP3034.
- Shared AST traversal (`Expr::collect_conjuncts` in `ast/expr.rs`) defines
  the same top-level `and` boundary for semantic checking and lowering.
- Compiler (`lowering/control_flow/conditions.rs`): conditions with `is` lower
  as short-circuit branches using the AP20.2 pattern tests; the tested value is
  evaluated once, and `while` re-evaluates the condition every iteration.
- Formatter (`emit/expr/patterns.rs`), closure capture, source-id remapping,
  constant classification, and the debugger (rejects `is`) handle the new
  expression; the VS Code grammar highlights `is`.
- Migration: no repository identifier was spelled `is`.

Coverage: `crates/fpas-sema/src/tests/stmt/is_tests.rs`,
`crates/fpas-parser/src/tests/expr/is_tests.rs`,
`crates/fpas-compiler/src/tests/control_flow/pattern_bindings.rs`,
`crates/fpas-fmt/tests/case_block_regressions.rs`, the lexer keyword tests,
`tests/runner/is_pattern_test.fpas`, `tests/runner/pattern_resolution_test.fpas`,
`crates/fpas-build/tests/import_aliases.rs`, and
`crates/fpas-cli/src/main_tests/projects/enum_variants.rs`.
