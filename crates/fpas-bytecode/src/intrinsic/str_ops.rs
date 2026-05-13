//! `Std.Str` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/str.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Str.*`.
///
/// **Documentation:** `docs/pascal/std/str.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum StrIntrinsic {
    Length = 20,
    ToUpper = 21,
    ToLower = 22,
    Trim = 23,
    Contains = 24,
    StartsWith = 25,
    EndsWith = 26,
    Substring = 27,
    IndexOf = 28,
    Replace = 29,
    Split = 30,
    Join = 31,
    IsNumeric = 32,
    /// `Std.Str.RepeatStr(S, N)` — repeat string N times; `N <= 0` returns `''`.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    Repeat = 200,
    /// `Std.Str.PadLeft(S, Width, PadChar)` — left-pad string to width.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    PadLeft = 201,
    /// `Std.Str.PadRight(S, Width, PadChar)` — right-pad string to width.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    PadRight = 202,
    /// `Std.Str.PadCenter(S, Width, PadChar)` — center-pad string to width.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    PadCenter = 203,
    /// `Std.Str.FromChar(C, N)` — create string of N copies of char C.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    FromChar = 204,
    /// `Std.Str.CharAt(S, Index)` — character at zero-based index.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    CharAt = 205,
    /// `Std.Str.SetCharAt(S, Index, C)` — return new string with char replaced.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    SetCharAt = 206,
    /// `Std.Str.Ord(C)` — Unicode code point of a char.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    Ord = 207,
    /// `Std.Str.Chr(N)` — char from Unicode code point.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    Chr = 208,
    /// `Std.Str.Insert(S, Index, Sub)` — insert substring at index.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    Insert = 209,
    /// `Std.Str.Delete(S, Index, Len)` — delete Len chars starting at Index.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    Delete = 210,
    /// `Std.Str.Reverse(S)` — reverse a string.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    Reverse = 211,
    /// `Std.Str.TrimLeft(S)` — trim leading whitespace.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    TrimLeft = 212,
    /// `Std.Str.TrimRight(S)` — trim trailing whitespace.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    TrimRight = 213,
    /// `Std.Str.LastIndexOf(S, Sub)` — last occurrence index or -1.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    LastIndexOf = 214,
    /// `Std.Str.Format(Template, ...)` — printf-style string formatting.
    ///
    /// Stack convention: template pushed first, then each arg, then arg count as integer.
    /// Specifiers: `%d` integer, `%f` real, `%s` string, `%%` literal percent.
    ///
    /// **Documentation:** `docs/pascal/std/str.md`
    Format = 242,
}
