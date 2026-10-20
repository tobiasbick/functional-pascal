//! Binary-text codecs expose fixed argument types and one nominal error family.

use super::{check_errors, check_ok};

#[test]
fn codecs_use_explicit_bytes_text_and_encoding_error_signatures() {
    for (encode, decode) in [
        ("Base64Encode", "Base64Decode"),
        ("Base64UrlEncode", "Base64UrlDecode"),
        ("HexEncode", "HexDecode"),
    ] {
        check_ok(&format!(
            "program T; uses Std.Encoding as Codec; begin const Encoded: result of (string, Codec.EncodingError) := Codec.{encode}(Bytes := [0, 255]); const Decoded: result of (array of integer, Codec.EncodingError) := Codec.{decode}(Text := ''); end."
        ));
        assert!(
            !check_errors(&format!(
                "program T; uses Std.Encoding as Codec; begin discard Codec.{encode}('text'); end."
            ))
            .is_empty()
        );
        assert!(
            !check_errors(&format!(
                "program T; uses Std.Encoding as Codec; begin discard Codec.{decode}([0]); end."
            ))
            .is_empty()
        );
    }
}
