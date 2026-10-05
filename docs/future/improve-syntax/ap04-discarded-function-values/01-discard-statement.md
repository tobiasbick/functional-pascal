# AP04.1: Discard statement

Package: [AP04: Discarded function values](README.md)

## Scope

Add the statement `discard Expression;` and reserve `discard`. Unused function
results remain accepted in this work package.

## Prerequisites

- AP02 (diagnostic codes).

## Implementation

- Lexer: reserve `discard`; diagnose its use as an identifier with a rename hint.
- Parser and AST: the `discard` statement.
- Sema: the operand must produce a value; reject procedure calls; reject task
  handles and values whose type contains task handles, pointing to
  `go Worker();`.
- Compiler: evaluate the operand exactly once and drop the value.
- Formatter and editor highlighting.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, `fpas-parser` statement parsing,
  `fpas-sema/src/check/stmt/`, `fpas-compiler` statement lowering, `fpas-fmt`,
  `editors/vscode/syntaxes/`.

## Migration

Rename any repository identifier spelled `discard`.

## Documentation

- `docs/specs/grammar.ebnf`, keyword list, a section on discarding values in
  the functions or statements documentation.

## Verification

- Tests: ordinary values, `Result`, `Option`, postfix chains, task handles,
  aggregates containing task handles, procedures, and exactly-once evaluation.
- Formatter round trip; VS Code grammar verification.
