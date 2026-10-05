# AP07.2: Std.Bits unit

Package: [AP07: Boolean rules](README.md)

## Scope

Add the `Std.Bits` unit with `BitAnd`, `BitOr`, `BitXor`, `BitNot`,
`ShiftLeft`, and `ShiftRight` (Q02). Existing operators remain until AP07.4.

## Prerequisites

None.

## Implementation

- Runtime functions in a unit-owned module under `crates/fpas-std/src/`.
- Registry signatures, intrinsic catalog entries, and generated declarations
  in `lib/api/Std/Bits.fpas`.
- Specify shift-count validation and right-shift fill behavior in the unit
  documentation before implementing it.

## Affected areas

- `crates/fpas-std/src/` (new `bits/` module), `crates/fpas-sema/src/std_registry/`,
  `crates/fpas-bytecode/src/intrinsic/`, `lib/api/Std/Bits.fpas`,
  `lib/stdlib.fpasprj`.

## Migration

None.

## Documentation

- New `Std.Bits` page under `docs/pascal/std/numeric/` and its index entry.

## Verification

- Rust tests: bit patterns, signed values, every valid shift count, invalid
  counts, wrong arity and types.
- FPAS test under `tests/stdlib/bits/` through the real runner.
