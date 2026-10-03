//! `Std.Bits` wire selectors. Documentation: `docs/pascal/std/numeric/bits.md`.
use num_enum::TryFromPrimitive;
documented_intrinsic_enum! {
/// Side-effect-free operations on 64-bit integer bit patterns.
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
