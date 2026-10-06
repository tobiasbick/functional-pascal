# AP07.2: Std.Bits unit

Package: [AP07: Boolean rules](README.md)

## Scope

Add the `Std.Bits` unit with `BitAnd`, `BitOr`, `BitXor`, `BitNot`,
`ShiftLeft`, and `ShiftRight` (Q02). AP07.4 owns integer operator removal
and consumer migration.

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
- Preserve the signed 64-bit value and error semantics of the former infix
  shifts. AP07.4 makes these functions the public integer-bit API. Both
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

## Prerequisites

None.

## Implementation

- Runtime functions in a unit-owned module under `crates/fpas-std/src/`.
- Registry signatures, intrinsic catalog entries, and generated declarations
  in `lib/api/Std/Bits.fpas`.
- Implement the agreed shift rules above and validate counts before applying
  a host shift operation. Reuse existing integer bit-operation handling where
  appropriate; runtime behavior must not depend on build overflow settings.

## Affected areas

- `crates/fpas-std/src/` (new `bits/` module), `crates/fpas-sema/src/std_registry/`,
  `crates/fpas-bytecode/src/intrinsic/`, `lib/api/Std/Bits.fpas`.

## Migration

None.

## Documentation

- New `Std.Bits` page under `docs/pascal/std/numeric/` and its index entry,
  including shift signatures, the agreed count range, arithmetic right shift,
  discarded left-shift bits, errors, and the boundary examples above.

## Verification

- Rust tests: bit patterns, signed values, every valid shift count, invalid
  counts, wrong arity and types.
- Shift boundaries: count zero, count 63, negative counts, counts of 64 or
  more, minimum/maximum integer values, sign extension of negative values,
  and discarded high bits without an arithmetic-overflow error. Check the
  required results above and exactly-once argument evaluation.
- FPAS test under `tests/stdlib/bits/` through the real runner.

## Implementation result

- The six functions are registered as an intrinsic unit, with generated editor
  declarations in `lib/api/Std/Bits.fpas` and a handbook in
  `docs/pascal/std/numeric/bits.md`.
- Integer primitives in `crates/fpas-std/src/bits/` implement the functions.
  Shift-count validation and runtime diagnostics are preserved. Debugger
  calls classify the functions as deterministic computation.
- `lib/stdlib.fpasprj` is unchanged: it controls source-defined standard units;
  intrinsic units are registered by `STD_UNITS_INTRINSIC` and do not compile
  their editor declaration stubs as library sources.
- AP07.4 removes integer operators and migrates their fixtures. Function
  results remain covered against fixed-width expected values.
