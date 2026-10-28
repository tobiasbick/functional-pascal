//! `Std.Os.Env` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/host/env.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Os.Env.*`.
///
/// **Documentation:** `docs/pascal/std/host/env.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum EnvIntrinsic {
    /// `Std.Os.Env.Get(Name)` - process environment lookup.
    ///
    /// **Documentation:** `docs/pascal/std/host/env.md`
    Get = 308,
    /// `Std.Os.Env.Exists(Name)` - process environment presence check.
    ///
    /// **Documentation:** `docs/pascal/std/host/env.md`
    Exists = 309,
}
}
