# `Std.Bits`

Bit operations on the signed 64-bit representation of `integer` values.
Use these functions for integer bits; the logical operators accept `boolean`.

## Importing and names

Import `uses Std.Bits;` before calling these functions. Both short names such
as `BitAnd` and qualified names such as `Std.Bits.BitAnd` follow the ordinary
[unit import rules](../../program-structure/units.md). Every parameter and
return value is an `integer`. Arguments are evaluated from left to right,
exactly once; both arguments of a binary function are always evaluated.

```pascal
program BitExample;

uses Std.Bits, Std.Console;

begin
  WriteLn(BitAnd(12, 10));
  WriteLn(ShiftRight(-8, 1));
end.
```

## Quick reference

Requires `uses Std.Bits;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `BitAnd(Left: integer; Right: integer): integer` | Keeps bits set in both operands. |
| function | `BitOr(Left: integer; Right: integer): integer` | Keeps bits set in either operand. |
| function | `BitXor(Left: integer; Right: integer): integer` | Keeps bits that differ between operands. |
| function | `BitNot(Value: integer): integer` | Complements all 64 bits. |
| function | `ShiftLeft(Value: integer; Count: integer): integer` | Shifts left, fills with zeros, and discards high bits; count must be `0..63`. |
| function | `ShiftRight(Value: integer; Count: integer): integer` | Shifts right with sign extension; count must be `0..63`. |

## `BitAnd`

**Parameters:** `Left` and `Right` are the two 64-bit patterns to intersect.
Each result bit is set only if the corresponding bit is set in both operands.
For example, `BitAnd(12, 10)` returns `8`, and `BitAnd(-1, Value)` returns
`Value`.

## `BitOr`

**Parameters:** `Left` and `Right` are the two 64-bit patterns to combine.
Each result bit is set if it is set in either operand. `BitOr(12, 10)` returns
`14`; `BitOr(-1, Value)` returns `-1`.

## `BitXor`

**Parameters:** `Left` and `Right` are the two 64-bit patterns to compare.
Each result bit is set if the corresponding bits differ. `BitXor(12, 10)`
returns `6`; `BitXor(Value, Value)` returns `0`.

## `BitNot`

**Parameter:** `Value` is the 64-bit pattern to complement. All 64 bits,
including the sign bit, are inverted. `BitNot(0)` returns `-1`, `BitNot(-1)`
returns `0`, and `BitNot(12)` returns `-13`.

## `ShiftLeft`

**Parameters:** `Value` is the pattern to shift; `Count` is the number of bit
positions, from `0` through `63` inclusive. Count zero returns `Value`.

Low bits are filled with zero and bits shifted past the highest position are
discarded. The remaining pattern is interpreted as a signed integer. Discarding
bits does not raise an arithmetic-overflow error: `ShiftLeft(1, 63)` returns
`-9223372036854775808`, and shifting that value left by one returns `0`.

## `ShiftRight`

**Parameters:** `Value` is the pattern to shift; `Count` is the number of bit
positions, from `0` through `63` inclusive. Count zero returns `Value`.

This is an arithmetic right shift: high positions are filled with the original
sign bit. Negative values stay negative. `ShiftRight(-8, 1)` returns `-4`, and
`ShiftRight(-1, Count)` returns `-1` for every valid count.

## Shift errors and boundaries

Both shift functions raise runtime error `FP5012` for a negative count or a
count of `64` or more. The diagnostic states the allowed range `0..63`.
Counts are never masked or reduced modulo 64. The rules are the same in
debug and release builds and in debugger watches.

| Call | Result |
|------|--------|
| `ShiftLeft(7, 0)` | `7` |
| `ShiftRight(-8, 1)` | `-4` |
| `ShiftRight(-1, 63)` | `-1` |
| `ShiftLeft(1, 63)` | `-9223372036854775808` |
| `ShiftLeft(9223372036854775807, 1)` | `-2` |
| `ShiftLeft(1, -1)` | Runtime error: count outside `0..63` |
| `ShiftRight(1, 64)` | Runtime error: count outside `0..63` |

## Implementation (contributors)

| Layer | Location |
|-------|----------|
| Integer primitives and intrinsic runtime | [`fpas-std/src/bits/`](../../../../crates/fpas-std/src/bits/mod.rs) |
| Semantic signatures | [`fpas-sema/src/std_registry/loaded/bits.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/bits.rs) |
| Intrinsic identifiers | [`fpas-bytecode/src/intrinsic/bits.rs`](../../../../crates/fpas-bytecode/src/intrinsic/bits.rs) |
| Compiler call mapping | [`fpas-compiler/src/intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Generated editor declarations | [`lib/api/Std/Bits.fpas`](../../../../lib/api/Std/Bits.fpas) |
| FPAS regressions | [`tests/stdlib/bits/`](../../../../tests/stdlib/bits) |

## See also

- [Numeric index](README.md)
- [Standard library index](../README.md)
- [Operators](../../language/basics/operators.md)
