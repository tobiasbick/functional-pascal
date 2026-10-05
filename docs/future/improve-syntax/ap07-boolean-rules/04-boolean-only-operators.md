# AP07.4: Boolean-only logical operators

Package: [AP07: Boolean rules](README.md)

## Scope

Restrict `and`, `or`, `xor`, and `not` to boolean operands and remove the
`shl` and `shr` operators and keywords.

## Prerequisites

- AP07.2 (`Std.Bits` provides the replacements).

## Implementation

- Sema: reject integer operands of the logical operators with a diagnostic
  naming the matching `Std.Bits` function.
- Lexer and parser: remove `shl`/`shr`; the words become ordinary identifiers.
  An infix `shl`/`shr` gets a diagnostic showing `ShiftLeft`/`ShiftRight`.
- Compiler and VM: remove integer logical and shift operator lowering.
- Editor highlighting drops the retired keywords.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, `fpas-parser` expression parsing,
  `fpas-sema/src/check/expr/operators.rs`, compiler expression lowering,
  `crates/fpas-vm/src/vm/value_ops/integer.rs`, `editors/vscode/syntaxes/`.

## Migration

- At planning time no `.fpas` source used `shl`, `shr`, `xor`, or integer
  `and`/`or`; recheck, and migrate Rust-embedded fixtures to `Std.Bits` calls.

## Documentation

- `docs/specs/grammar.ebnf`, `operators.md`, keyword list, debugger expression
  documentation.

## Verification

- Rejection tests for each operator with integer operands and for infix shifts.
- Lexer tests for retired keywords used as identifiers.
- VS Code grammar verification.
