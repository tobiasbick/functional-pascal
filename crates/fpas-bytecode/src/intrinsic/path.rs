//! `Std.Path` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/path.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Path.*`.
///
/// **Documentation:** `docs/pascal/std/path.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum PathIntrinsic {
    /// `Std.Path.Join(Segments)` — join path segments with the platform separator.
    ///
    /// **Documentation:** `docs/pascal/std/path.md`
    Join = 310,
    /// `Std.Path.BaseName(Path)` — final path component.
    ///
    /// **Documentation:** `docs/pascal/std/path.md`
    BaseName = 311,
    /// `Std.Path.DirName(Path)` — parent path without the final component.
    ///
    /// **Documentation:** `docs/pascal/std/path.md`
    DirName = 312,
    /// `Std.Path.Extension(Path)` — file extension without a leading dot.
    ///
    /// **Documentation:** `docs/pascal/std/path.md`
    Extension = 313,
    /// `Std.Path.Normalize(Path)` — normalize separators and `.` / `..` components.
    ///
    /// **Documentation:** `docs/pascal/std/path.md`
    Normalize = 314,
}
