# AP07.2: Std.Bits unit

Package: [AP07: Boolean rules](README.md)

Status: complete.

## Result

`uses Std.Bits;` exposes `BitAnd`, `BitOr`, `BitXor`, `BitNot`, `ShiftLeft`,
and `ShiftRight` for signed 64-bit integers. Integer bit operations use these
functions. Runtime primitives live in `crates/fpas-std/src/bits/`; intrinsic
registration supplies `lib/api/Std/Bits.fpas`. Editor stubs are not source-library
units. Debugger calls classify these operations as deterministic computation.

## Shift decisions (agreed)

- `ShiftLeft(Value: integer; Count: integer): integer` and
  `ShiftRight(Value: integer; Count: integer): integer` operate on FPAS's
  signed 64-bit integer representation.
- `Count` must be between **0 and 63 inclusive**. A zero count returns the
  original value. Negative counts and counts of 64 or more raise a runtime
  error stating the allowed range; they are not masked or reduced modulo 64.
- `ShiftRight` is an arithmetic right shift: high bits are filled with the
  original sign bit. For example, `ShiftRight(-8, 1)` returns `-4`, and
  shifting `-1` by any valid count returns `-1`.
- `ShiftLeft` shifts the 64-bit pattern, fills low bits with zero, and discards
  bits shifted beyond the high end. Interpret the remaining pattern as a
  signed integer; do not raise an arithmetic-overflow error. For example,
  `ShiftLeft(1, 63)` returns the minimum integer value.
- These functions are the public integer-bit API. Both
  arguments use ordinary left-to-right, exactly-once evaluation.

These rules settle shift-count validation and right-shift fill behavior.
The runtime implementation and the new unit documentation use the same rules.

| Call | Required result |
| --- | --- |
| `ShiftLeft(7, 0)` | `7` |
| `ShiftRight(-8, 1)` | `-4` |
| `ShiftRight(-1, 63)` | `-1` |
| `ShiftLeft(1, 63)` | `-9223372036854775808` |
| `ShiftLeft(1, -1)` | Runtime error: count outside `0..63` |
| `ShiftRight(1, 64)` | Runtime error: count outside `0..63` |

## Regression coverage

Rust and FPAS tests cover fixed-width results, signed values, all valid shift
counts, invalid counts, arity/types and argument evaluation.
See [Std.Bits](../../../pascal/std/numeric/bits.md).
