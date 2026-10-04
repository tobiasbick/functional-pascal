# Std.Bits

Side-effect-free operations on the 64-bit patterns of signed `integer` values.
Import with `uses Std.Bits as Bits;` and qualify each call through that alias.

## Quick reference

| Kind | Signature | Description |
|------|-----------|-------------|
| function | `BitAnd(Left: integer; Right: integer): integer` | Set bits present in both inputs |
| function | `BitOr(Left: integer; Right: integer): integer` | Set bits present in either input |
| function | `BitXor(Left: integer; Right: integer): integer` | Set bits present in exactly one input |
| function | `BitNot(Value: integer): integer` | Invert every bit |
| function | `ShiftLeft(Value: integer; Count: integer): integer` | Shift left, discarding shifted-out bits |
| function | `ShiftRight(Value: integer; Count: integer): integer` | Shift right, filling with zero bits |

## BitAnd

`Bits.BitAnd(12, 10)` returns `8`. The sign bit participates like every other bit;
`Bits.BitAnd(Value, -1)` returns `Value`.

## BitOr

`Bits.BitOr(12, 10)` returns `14`. Combining any value with `-1` returns `-1`.

## BitXor

`Bits.BitXor(12, 10)` returns `6`. Combining a value with itself returns `0`.

## BitNot

`Bits.BitNot(0)` returns `-1`; `Bits.BitNot(-1)` returns `0`.

## ShiftLeft

Counts from `0` through `63` are valid. A zero count preserves the input.
Shifted-out bits are discarded, with no integer arithmetic overflow check:
`Bits.ShiftLeft(1, 63)` returns `-9223372036854775808`.

## ShiftRight

The shift always zero-fills, including negative inputs:
`Bits.ShiftRight(-1, 1)` returns `9223372036854775807` and
`Bits.ShiftRight(-1, 63)` returns `1`. Count zero preserves the original pattern.

For both shifts, counts below `0` or above `63` fail at runtime with a numeric
domain diagnostic, including when `Value` is zero. Arguments must have type
`integer` and evaluate once, left to right. Bit calls are ordinary function calls
and are not accepted in compile-time-only constant initializers.

## Example

```pascal
program Flags;
uses Std.Bits as Bits;
uses Std.Console as Console;
begin
  const Flags: integer := Bits.BitOr(1, 4);
  Console.WriteLn(Bits.BitAnd(Flags, 4) <> 0);
end program;
```

## Implementation (contributors)

| Layer | Owner |
|-------|-------|
| Signatures | `crates/fpas-sema/src/std_registry/loaded/bits.rs` |
| Call mapping | `crates/fpas-compiler/src/intrinsic_catalog.rs` |
| Wire selectors | `crates/fpas-bytecode/src/intrinsic/bits.rs` |
| Runtime and unit tests | `crates/fpas-std/src/bits/` |
| Execution regression | `tests/stdlib/bits/bit_patterns_test.fpas` |
| Editor declarations | `lib/api/Std/Bits.fpas` (generated) |

## See also

- [Numeric units](README.md)
- [Standard library](../README.md)
- [Operators](../../language/basics/operators.md)
