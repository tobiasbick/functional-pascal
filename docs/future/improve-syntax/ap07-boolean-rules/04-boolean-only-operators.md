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
  IR operator categories, bytecode opcodes and validation, constant folding,
  `crates/fpas-vm/src/vm/value_ops/`, debugger expression validation, formatter
  operator tables, and `editors/vscode/syntaxes/` with grammar verification.

## Migration

- Integer operations in Rust-embedded compiler and debugger fixtures use
  `Std.Bits` calls. The short-circuit FPAS regression uses `BitAnd` and `BitOr`
  for its eager integer-argument checks. Boolean `xor` remains valid, including
  the typed scalar benchmark.
- VM scalar tests cover the remaining opcodes; invalid shift-count tests use
  the `Std.Bits` intrinsic. Compiler bit-function tests compare fixed-width
  expected values rather than removed FPAS operators.

## Documentation

- `docs/specs/grammar.ebnf`, `operators.md`, keyword list, debugger expression
  documentation.

## Verification

- Rejection tests for each operator with integer operands and for infix shifts.
- Lexer tests for retired keywords used as identifiers.
- VS Code grammar verification.

## Implementation result

- Logical operators accept only boolean operands. Integer-only expressions
  suggest the corresponding `Std.Bits` function and import; mixed operands
  explain the required Boolean types without suggesting a mixed bit call.
- `shl` and `shr` are identifiers in every ASCII letter case. Retired infix
  syntax receives a function hint and recovers to the next statement.
- Integer bit and shift operator paths are removed from parser AST, lowering,
  IR, constant folding, bytecode, VM dispatch, and debugger evaluation. Retired
  bytecode IDs are rejected; surviving IDs retain their assignments.
- Debugger validation rejects known non-Boolean operands before evaluation,
  including skipped literals, and runtime checks reject resolved integer
  logical values. Boolean short-circuit budgets remain intact.
- IR operator kinds and type categories live in the focused
  `crates/fpas-ir/src/instruction/operators.rs` module.
