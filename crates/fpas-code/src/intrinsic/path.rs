//! `Std.Io.Path` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/host/path.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Io.Path.*`.
///
/// **Documentation:** `docs/pascal/std/host/path.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum PathIntrinsic {
    /// `Std.Io.Path.Join(Segments)` — join path segments with the platform separator.
    ///
    /// **Documentation:** `docs/pascal/std/host/path.md`
    Join = 310,
    /// `Std.Io.Path.BaseName(Path)` — final path component.
    ///
    /// **Documentation:** `docs/pascal/std/host/path.md`
    BaseName = 311,
    /// `Std.Io.Path.DirName(Path)` — parent path without the final component.
    ///
    /// **Documentation:** `docs/pascal/std/host/path.md`
    DirName = 312,
    /// `Std.Io.Path.Extension(Path)` — file extension without a leading dot.
    ///
    /// **Documentation:** `docs/pascal/std/host/path.md`
    Extension = 313,
    /// `Std.Io.Path.Normalize(Path)` — normalize separators and `.` / `..` components.
    ///
    /// **Documentation:** `docs/pascal/std/host/path.md`
    Normalize = 314,
    /// `Std.Io.Path.IsAbsolute(Path)` — test the host platform's lexical absolute-path rules.
    ///
    /// **Documentation:** `docs/pascal/std/host/path.md`
    IsAbsolute = 331,
    /// `Std.Io.Path.RelativeTo(Target, Base)` — optional lexical relative path from Base to Target.
    ///
    /// **Documentation:** `docs/pascal/std/host/path.md`
    RelativeTo = 332,
    /// `Std.Io.Path.ChangeExtension(Path, Extension)` — optionally replace the final suffix.
    ///
    /// **Documentation:** `docs/pascal/std/host/path.md`
    ChangeExtension = 333,
}
}
