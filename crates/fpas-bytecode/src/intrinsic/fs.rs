//! `Std.Fs` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Fs.*`.
///
/// **Documentation:** `docs/pascal/std/host/fs.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum FsIntrinsic {
    /// `Std.Fs.ReadText(Path)` - read a UTF-8 text file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ReadText = 315,
    /// `Std.Fs.WriteText(Path, Text)` - write UTF-8 text to a file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    WriteText = 316,
    /// `Std.Fs.Exists(Path)` - check whether a path exists.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    Exists = 317,
    /// `Std.Fs.IsFile(Path)` - check whether a path is a regular file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    IsFile = 318,
    /// `Std.Fs.IsDir(Path)` - check whether a path is a directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    IsDir = 319,
    /// `Std.Fs.CreateDir(Path)` - create a single directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CreateDir = 320,
    /// `Std.Fs.Glob(Pattern)` - expand a glob pattern to matching file paths.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    Glob = 343,
    /// `Std.Fs.WriteTextAtomic(Path, Text)` - durably replace a UTF-8 text file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    WriteTextAtomic = 517,
}
