//! Byte-domain failures, hex interoperability and codec resource boundaries.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "codec tests inspect exact outcomes"
)]

use super::*;

fn location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn buffer(bytes: &[u8]) -> Value {
    Value::native_bytes(bytes.to_vec())
}

fn invoke(operation: EncodingIntrinsic, argument: Value) -> Value {
    crate::stdlib::run_intrinsic_borrowed(
        Intrinsic::Encoding(operation),
        &[argument],
        location(),
        &crate::stdlib::intrinsics::TEST_AGGREGATES,
    )
    .unwrap()
    .unwrap()
}

fn assert_failure(actual: Value, variant: &str, fields: &[Value]) {
    let Value::ResultError(error) = actual else {
        panic!("expected Error, got {actual:?}")
    };
    let Value::Enum(error) = error.as_ref() else {
        panic!("expected native encoding error")
    };
    assert_eq!(error.body().layout.type_name, "Std.Encoding.EncodingError");
    assert_eq!(error.body().layout.variant, variant);
    assert_eq!(error.body().values, fields);
}

#[test]
fn hex_encodes_every_byte_canonically_and_decodes_either_case() {
    let bytes: Vec<u8> = (0..=255).collect();
    let expected = bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    assert_eq!(hex::encode(&bytes), expected);
    assert_eq!(hex::decode(&expected).unwrap(), bytes);
    assert_eq!(hex::decode(&expected.to_lowercase()).unwrap(), bytes);
    assert_eq!(hex::decode("aBcD").unwrap(), [171, 205]);
    assert_eq!(hex::encode(&[]), "");
    assert!(hex::decode("").unwrap().is_empty());
}

#[test]
fn hex_requires_complete_ascii_byte_pairs_and_reports_input_byte_offsets() {
    for text in ["0", "ABC", "0x0", "é0"] {
        assert_eq!(
            hex::decode(text),
            Err(CodecFailure::InvalidLength { length: text.len() })
        );
    }
    for (text, offset) in [
        ("0x00", 1),
        ("0G", 1),
        ("AA B", 2),
        ("AA\nB", 2),
        ("00_0", 2),
        ("$0", 0),
        ("AAé", 2),
        ("0\0", 1),
    ] {
        assert_eq!(
            hex::decode(text),
            Err(CodecFailure::InvalidCharacter { offset })
        );
    }
}

#[test]
fn codec_dispatch_preserves_inputs_and_returns_typed_successes() {
    let arguments = [buffer(&[0, 15, 255])];
    let before = arguments.clone();
    let result = crate::stdlib::run_intrinsic_borrowed(
        Intrinsic::Encoding(EncodingIntrinsic::HexEncode),
        &arguments,
        location(),
        &crate::stdlib::intrinsics::TEST_AGGREGATES,
    )
    .unwrap();
    assert_eq!(arguments, before);
    assert_eq!(result, Some(Value::result_ok(Value::Str("000FFF".into()))));
    for operation in [
        EncodingIntrinsic::Base64Encode,
        EncodingIntrinsic::Base64UrlEncode,
        EncodingIntrinsic::HexEncode,
    ] {
        assert_eq!(
            invoke(operation, buffer(&[])),
            Value::result_ok(Value::Str("".into()))
        );
    }
    for operation in [
        EncodingIntrinsic::Base64Decode,
        EncodingIntrinsic::Base64UrlDecode,
        EncodingIntrinsic::HexDecode,
    ] {
        assert_eq!(
            invoke(operation, Value::Str("".into())),
            Value::result_ok(buffer(&[]))
        );
    }
    assert_eq!(
        invoke(EncodingIntrinsic::Base64Decode, Value::Str("AA//".into())),
        Value::result_ok(buffer(&[0, 15, 255]))
    );
    assert_eq!(
        invoke(
            EncodingIntrinsic::Base64UrlDecode,
            Value::Str("AA__".into())
        ),
        Value::result_ok(buffer(&[0, 15, 255]))
    );
}

#[test]
fn encoders_require_native_buffers_without_implicit_array_conversion() {
    for operation in [
        EncodingIntrinsic::Base64Encode,
        EncodingIntrinsic::Base64UrlEncode,
        EncodingIntrinsic::HexEncode,
    ] {
        for invalid in [-1, 256, i64::MIN, i64::MAX] {
            let argument = Value::Array(vec![Value::Integer(0), Value::Integer(invalid)].into());
            let error = crate::stdlib::run_intrinsic_borrowed(
                Intrinsic::Encoding(operation),
                &[argument],
                location(),
                &crate::stdlib::intrinsics::TEST_AGGREGATES,
            )
            .unwrap_err();
            assert_eq!(
                error.code,
                fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH
            );
        }
    }
}

#[test]
fn malformed_decoder_results_use_native_error_variants_and_payloads() {
    for (operation, text, variant, offset) in [
        (EncodingIntrinsic::HexDecode, "0G", "InvalidCharacter", 1),
        (EncodingIntrinsic::HexDecode, "AAé", "InvalidCharacter", 2),
        (EncodingIntrinsic::HexDecode, "A", "InvalidLength", 1),
        (EncodingIntrinsic::Base64Decode, "AA=A", "InvalidPadding", 2),
        (EncodingIntrinsic::Base64Decode, "Zh==", "NonCanonical", 1),
        (
            EncodingIntrinsic::Base64UrlDecode,
            "Zg==",
            "InvalidPadding",
            2,
        ),
    ] {
        assert_failure(
            invoke(operation, Value::Str(text.into())),
            variant,
            &[Value::Integer(offset)],
        );
    }
}

#[test]
fn binary_limit_round_trips_through_each_text_expansion() {
    let bytes: Vec<u8> = (0..MAX_BYTES).map(|n| (n % 256) as u8).collect();
    for (url, expected_len) in [(false, 1_398_104), (true, 1_398_102)] {
        let text = base64::encode(&bytes, url);
        assert_eq!(text.len(), expected_len);
        assert_eq!(base64::decode(&text, url).unwrap(), bytes);
    }
    let text = hex::encode(&bytes);
    assert_eq!(text.len(), 2_097_152);
    assert_eq!(hex::decode(&text).unwrap(), bytes);
}

#[test]
fn binary_and_text_limits_reject_before_byte_or_text_content_validation() {
    let values = vec![0; MAX_BYTES + 1];
    assert_failure(
        invoke(EncodingIntrinsic::HexEncode, Value::native_bytes(values)),
        "LimitExceeded",
        &[Value::Integer(MAX_BYTES as i64)],
    );
    assert_eq!(
        hex::decode(&"?".repeat(MAX_BYTES * 2 + 1)),
        Err(CodecFailure::LimitExceeded)
    );
    assert_eq!(
        base64::decode(&"?".repeat(1_398_105), false),
        Err(CodecFailure::LimitExceeded)
    );
    assert_eq!(
        base64::decode(&"?".repeat(1_398_103), true),
        Err(CodecFailure::LimitExceeded)
    );
    // This text meets the encoded limit but would decode to two bytes over the binary limit.
    assert_eq!(
        base64::decode(&"AAAA".repeat(349_526), false),
        Err(CodecFailure::LimitExceeded)
    );
}
