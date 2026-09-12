//! `Std.Crypto` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/cryptography/crypto.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Crypto.*`.
///
/// **Documentation:** `docs/pascal/std/cryptography/crypto.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum CryptoIntrinsic {
    /// `Std.Crypto.RandomBytes(Count)` - bytes from the operating-system random source.
    RandomBytes = 611,
    /// `Std.Crypto.RandomInt(Lo, Hi)` - unbiased integer from the operating-system random source.
    RandomInt = 612,
}
}
