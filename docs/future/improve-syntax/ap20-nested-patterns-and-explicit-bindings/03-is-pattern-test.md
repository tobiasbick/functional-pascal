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
