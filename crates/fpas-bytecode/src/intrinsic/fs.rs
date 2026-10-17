//! `Std.Fs` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
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
    /// `Std.Fs.CreateDirAll(Path)` - create a directory and missing parents.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CreateDirAll = 613,
    /// `Std.Fs.Glob(Pattern)` - expand a glob pattern to matching file paths.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    Glob = 343,
    /// `Std.Fs.ReadDir(Path)` - list immediate files and directories in stable order.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ReadDir = 620,
    /// `Std.Fs.WriteTextAtomic(Path, Text)` - durably replace a UTF-8 text file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    WriteTextAtomic = 517,
    /// `Std.Fs.DeleteFile(Path)` - remove one file entry.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    DeleteFile = 567,
    /// `Std.Fs.DeleteDir(Path)` - remove one empty directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    DeleteDir = 621,
    /// `Std.Fs.DeleteDirAll(Path)` - remove a directory and all of its contents.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    DeleteDirAll = 622,
    /// `Std.Fs.Rename(Source, Target)` - rename or move a file or directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    Rename = 623,
    /// `Std.Fs.CopyFile(Source, Target)` - copy one file, replacing the target.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CopyFile = 624,
    /// `Std.Fs.AppendText(Path, Text)` - append UTF-8 text, creating the file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    AppendText = 625,
    /// `Std.Fs.FileSize(Path)` - size in bytes of one regular file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    FileSize = 626,
    /// `Std.Fs.ModifiedTime(Path)` - last modification time in Unix milliseconds.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ModifiedTime = 627,
    /// `Std.Fs.IsSymLink(Path)` - check whether a path is a symbolic link.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    IsSymLink = 628,
    /// `Std.Fs.CurrentDir()` - current working directory of the process.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CurrentDir = 629,
    /// `Std.Fs.SetCurrentDir(Path)` - change the process working directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    SetCurrentDir = 630,
    /// `Std.Fs.TempDir()` - host temporary directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    TempDir = 631,
    /// `Std.Fs.CreateTempDir()` - create a uniquely named temporary directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CreateTempDir = 632,
    /// `Std.Fs.CreateTempFile()` - create a uniquely named empty temporary file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CreateTempFile = 633,
    /// `Std.Fs.ReadDirAll(Path)` - list all entries below a directory recursively.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ReadDirAll = 634,
    /// `Std.Fs.CopyDirAll(Source, Target)` - copy a directory tree.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CopyDirAll = 635,
}
}
