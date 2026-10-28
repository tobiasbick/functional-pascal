//! `Std.Bits` intrinsic discriminants.
//!
//! Documentation: `docs/pascal/std/numeric/bits.md`.

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Signed 64-bit bit operations supplied by `Std.Bits`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum BitsIntrinsic {
    BitAnd = 614,
    BitOr = 615,
    BitXor = 616,
    BitNot = 617,
    ShiftLeft = 618,
    ShiftRight = 619,
}
}
