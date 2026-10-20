//! `Std.Encoding` names and registry inventory.

std_symbol!(STD_ENCODING_BASE64_ENCODE = "Std.Encoding.Base64Encode");
std_symbol!(STD_ENCODING_BASE64_DECODE = "Std.Encoding.Base64Decode");
std_symbol!(STD_ENCODING_BASE64_URL_ENCODE = "Std.Encoding.Base64UrlEncode");
std_symbol!(STD_ENCODING_BASE64_URL_DECODE = "Std.Encoding.Base64UrlDecode");
std_symbol!(STD_ENCODING_HEX_ENCODE = "Std.Encoding.HexEncode");
std_symbol!(STD_ENCODING_HEX_DECODE = "Std.Encoding.HexDecode");
std_symbol!(STD_ENCODING_ERROR = "Std.Encoding.EncodingError");

pub(in crate::std_units) const STD_ENCODING_SYMBOLS: &[&str] = &[
    STD_ENCODING_BASE64_ENCODE,
    STD_ENCODING_BASE64_DECODE,
    STD_ENCODING_BASE64_URL_ENCODE,
    STD_ENCODING_BASE64_URL_DECODE,
    STD_ENCODING_HEX_ENCODE,
    STD_ENCODING_HEX_DECODE,
];
