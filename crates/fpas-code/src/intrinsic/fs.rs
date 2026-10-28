//! `Std.Io.Fs` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Io.Fs.*`.
///
/// **Documentation:** `docs/pascal/std/host/fs.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum FsIntrinsic {
    /// `Std.Io.Fs.Open` — bounded VM-owned file operation.
    Open = 710,
    /// `Std.Io.Fs.Close` — bounded VM-owned file operation.
    Close = 711,
    /// `Std.Io.Fs.Read` — bounded VM-owned file operation.
    Read = 712,
    /// `Std.Io.Fs.Write` — bounded VM-owned file operation.
    Write = 713,
    /// `Std.Io.Fs.ReadLine` — bounded VM-owned file operation.
    ReadLine = 714,
    /// `Std.Io.Fs.Seek` — bounded VM-owned file operation.
    Seek = 715,
    /// `Std.Io.Fs.Tell` — bounded VM-owned file operation.
    Tell = 716,
    /// `Std.Io.Fs.Flush` — bounded VM-owned file operation.
    Flush = 717,
    /// `Std.Io.Fs.ReadWithCancellation` — bounded VM-owned file operation.
    ReadWithCancellation = 718,
    /// `Std.Io.Fs.WriteWithCancellation` — bounded VM-owned file operation.
    WriteWithCancellation = 719,
    /// `Std.Io.Fs.ReadLineWithCancellation` — bounded VM-owned file operation.
    ReadLineWithCancellation = 720,

    /// `Std.Io.Fs.ReadBytes(Path)` - read a bounded binary file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ReadBytes = 708,
    /// `Std.Io.Fs.WriteBytes(Path, Bytes)` - write a bounded array of byte integers.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    WriteBytes = 709,
    /// `Std.Io.Fs.ReadText(Path)` - read a UTF-8 text file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ReadText = 315,
    /// `Std.Io.Fs.WriteText(Path, Text)` - write UTF-8 text to a file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    WriteText = 316,
    /// `Std.Io.Fs.Exists(Path)` - check whether a path exists.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    Exists = 317,
    /// `Std.Io.Fs.IsFile(Path)` - check whether a path is a regular file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    IsFile = 318,
    /// `Std.Io.Fs.IsDir(Path)` - check whether a path is a directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    IsDir = 319,
    /// `Std.Io.Fs.CreateDir(Path)` - create a single directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CreateDir = 320,
    /// `Std.Io.Fs.CreateDirAll(Path)` - create a directory and missing parents.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CreateDirAll = 613,
    /// `Std.Io.Fs.Glob(Pattern)` - expand a glob pattern to matching file paths.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    Glob = 343,
    /// `Std.Io.Fs.ReadDir(Path)` - list immediate files and directories in stable order.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ReadDir = 620,
    /// `Std.Io.Fs.WriteTextAtomic(Path, Text)` - durably replace a UTF-8 text file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    WriteTextAtomic = 517,
    /// `Std.Io.Fs.DeleteFile(Path)` - remove one file entry.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    DeleteFile = 567,
    /// `Std.Io.Fs.DeleteDir(Path)` - remove one empty directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    DeleteDir = 621,
    /// `Std.Io.Fs.DeleteDirAll(Path)` - remove a directory and all of its contents.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    DeleteDirAll = 622,
    /// `Std.Io.Fs.Rename(Source, Target)` - rename or move a file or directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    Rename = 623,
    /// `Std.Io.Fs.CopyFile(Source, Target)` - copy one file, replacing the target.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CopyFile = 624,
    /// `Std.Io.Fs.AppendText(Path, Text)` - append UTF-8 text, creating the file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    AppendText = 625,
    /// `Std.Io.Fs.FileSize(Path)` - size in bytes of one regular file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    FileSize = 626,
    /// `Std.Io.Fs.ModifiedTime(Path)` - last modification time in Unix milliseconds.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ModifiedTime = 627,
    /// `Std.Io.Fs.IsSymLink(Path)` - check whether a path is a symbolic link.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    IsSymLink = 628,
    /// `Std.Io.Fs.CurrentDir()` - current working directory of the process.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CurrentDir = 629,
    /// `Std.Io.Fs.SetCurrentDir(Path)` - change the process working directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    SetCurrentDir = 630,
    /// `Std.Io.Fs.TempDir()` - host temporary directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    TempDir = 631,
    /// `Std.Io.Fs.CreateTempDir()` - create a uniquely named temporary directory.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CreateTempDir = 632,
    /// `Std.Io.Fs.CreateTempFile()` - create a uniquely named empty temporary file.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CreateTempFile = 633,
    /// `Std.Io.Fs.ReadDirAll(Path)` - list all entries below a directory recursively.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    ReadDirAll = 634,
    /// `Std.Io.Fs.CopyDirAll(Source, Target)` - copy a directory tree.
    ///
    /// **Documentation:** `docs/pascal/std/host/fs.md`
    CopyDirAll = 635,
}
}
