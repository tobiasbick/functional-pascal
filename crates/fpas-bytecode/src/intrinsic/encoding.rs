//! Stable selectors for bounded binary-text codecs in `Std.Encoding`.

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Encoding`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum EncodingIntrinsic {
    /// `Std.Encoding.Base64Encode(Bytes)` binary-text conversion.
    Base64Encode = 702,
    /// `Std.Encoding.Base64Decode(Text)` binary-text conversion.
    Base64Decode = 703,
    /// `Std.Encoding.Base64UrlEncode(Bytes)` binary-text conversion.
    Base64UrlEncode = 704,
    /// `Std.Encoding.Base64UrlDecode(Text)` binary-text conversion.
    Base64UrlDecode = 705,
    /// `Std.Encoding.HexEncode(Bytes)` binary-text conversion.
    HexEncode = 706,
    /// `Std.Encoding.HexDecode(Text)` binary-text conversion.
    HexDecode = 707,
}
}
