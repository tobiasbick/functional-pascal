# AP07.4: Boolean-only logical operators

Package: [AP07: Boolean rules](README.md)

Status: complete.

## Result

`and`, `or`, `xor`, and `not` accept Boolean operands only. Integer operands
receive the matching `Std.Bits` function/import hint; mixed operands receive a
Boolean type diagnostic. `shl` and `shr` are ordinary identifiers; infix uses
are rejected with `ShiftLeft`/`ShiftRight` hints.

Integer logical and infix-shift paths are absent from AST, lowering, IR,
constant folding, bytecode dispatch and debugger evaluation. Retired bytecode
IDs are rejected. Editor highlighting uses the current keyword set.

## Regression coverage

Tests cover every rejected integer operator, case-insensitive identifiers,
recovery, fixed-width bit functions, debugger validation, budgets and grammar.
See [operators](../../../pascal/language/basics/operators.md).
