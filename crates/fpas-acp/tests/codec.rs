//! Independent wire documents protect numeric precision, presence and SDK normalization.

use fpas_acp::{
    CodecError, ErrorKind, RawJson, UInt64, codec,
    protocol::{self, v1},
};

fn kind<T>(result: Result<T, CodecError>) -> ErrorKind {
    result.err().expect("expected rejected value").kind
}

#[test]
fn raw_json_preserves_numbers_spelling_order_and_rejects_decoded_duplicate_keys() {
    let text = r#"{"z":9007199254740993,"a":[18446744073709551615,18446744073709551616,1e400,-0],"$serde_json::private::Number":"ordinary key"}"#;
    let raw = RawJson::parse(text).expect("arbitrary JSON numbers are protocol data");
    assert_eq!(raw.text(), text);
    for duplicate in [r#"{"a":1,"\u0061":2}"#, r#"{"x":[{"n":1,"n":2}]}"#] {
        assert_eq!(kind(RawJson::parse(duplicate)), ErrorKind::DuplicateKey);
    }
    for invalid in ["", "{} []", "NaN", "1.", "[1,]", "\"\\ud800\""] {
        assert_eq!(kind(RawJson::parse(invalid)), ErrorKind::InvalidMessage);
    }
    let nested = RawJson::parse(r#"{"nested":"\ud800"}"#).unwrap_err();
    assert_eq!(nested.kind, ErrorKind::InvalidMessage);
    assert_eq!((nested.line, nested.column), (None, None));
    assert_eq!(
        kind(RawJson::parse_bounded(text, text.len() - 1)),
        ErrorKind::LimitExceeded
    );
    assert!(RawJson::parse_bounded(text, text.len()).is_ok());
    assert!(!format!("{raw:?}").contains("9007199254740993"));
}

#[test]
fn unsigned_counters_are_exact_and_integer_conversions_are_checked() {
    for text in [
        "0",
        "9007199254740993",
        "9223372036854775807",
        "18446744073709551615",
    ] {
        assert_eq!(UInt64::parse(text).unwrap().decimal(), text);
    }
    assert_eq!(UInt64::parse("00042").unwrap().decimal(), "42");
    assert_eq!(
        UInt64::from_integer(i64::MAX)
            .unwrap()
            .to_integer()
            .unwrap(),
        i64::MAX
    );
    for text in ["", "-1", "+1", "1.0", "1e3", "１２", "18446744073709551616"] {
        assert_eq!(kind(UInt64::parse(text)), ErrorKind::InvalidInteger);
    }
    assert_eq!(kind(UInt64::canonical("00")), ErrorKind::InvalidInteger);
    assert_eq!(kind(UInt64::from_integer(-1)), ErrorKind::InvalidInteger);
    assert_eq!(
        kind(UInt64::parse("18446744073709551615").unwrap().to_integer()),
        ErrorKind::InvalidInteger
    );
}

#[test]
fn sdk_decode_keeps_unknown_input_and_pinned_optional_field_defaults() {
    let text = r#"{"protocolVersion":1,"clientInfo":false,"future":{"exact":18446744073709551616},"_meta":{"n":9007199254740993}}"#;
    let decoded = codec::decode::<v1::InitializeRequest>(RawJson::parse(text).unwrap()).unwrap();
    assert_eq!(
        decoded.value.protocol_version,
        protocol::ProtocolVersion::V1
    );
    assert!(decoded.value.client_info.is_none());
    assert_eq!(decoded.raw.text(), text);
    let encoded = codec::encode(&decoded.value, 4096).unwrap();
    assert!(
        encoded.text().contains("9007199254740993"),
        "{}",
        encoded.text()
    );
    assert_eq!(
        kind(codec::decode::<v1::InitializeRequest>(
            RawJson::parse("{}").unwrap()
        )),
        ErrorKind::InvalidMessage
    );
}

#[test]
fn known_nested_float_fields_are_never_silently_serialized_as_null() {
    #[derive(serde::Serialize)]
    struct Number {
        value: Option<Vec<f64>>,
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            kind(codec::encode(
                &Number {
                    value: Some(vec![value])
                },
                4096
            )),
            ErrorKind::NonFinite
        );
    }
    assert_eq!(
        kind(codec::encode(&"larger than budget", 2)),
        ErrorKind::LimitExceeded
    );
    assert_eq!(
        codec::encode(
            &Number {
                value: Some(vec![1.5])
            },
            4096
        )
        .unwrap()
        .text(),
        r#"{"value":[1.5]}"#
    );
}

#[test]
fn patches_preserve_missing_null_and_empty_replacement() {
    for (text, expected) in [
        (
            r#"{"messageId":"message"}"#,
            protocol::MaybeUndefined::Undefined,
        ),
        (
            r#"{"messageId":"message","content":null}"#,
            protocol::MaybeUndefined::Null,
        ),
        (
            r#"{"messageId":"message","content":[]}"#,
            protocol::MaybeUndefined::Value(vec![]),
        ),
    ] {
        let decoded = codec::decode::<v1::SessionMessage>(RawJson::parse(text).unwrap()).unwrap();
        assert_eq!(decoded.value.content, expected);
    }
}

#[test]
fn future_v2_content_remains_unknown_with_its_original_data() {
    let text = r#"{"type":"_future","payload":{"exact":18446744073709551616,"spelling":1e400}}"#;
    let decoded =
        codec::decode::<protocol::v2::ContentBlock>(RawJson::parse(text).unwrap()).unwrap();
    assert!(matches!(
        decoded.value,
        protocol::v2::ContentBlock::Other(_)
    ));
    assert_eq!(decoded.raw.text(), text);
    let payload = decoded.raw.member("payload").unwrap().unwrap();
    assert_eq!(
        payload.text(),
        r#"{"exact":18446744073709551616,"spelling":1e400}"#
    );
    assert_eq!(
        codec::encode(&payload, 4096).unwrap().text(),
        payload.text()
    );
    assert!(decoded.raw.member("absent").unwrap().is_none());
}

#[test]
fn untagged_decimal_values_and_raw_arbitrary_numbers_can_share_one_document() {
    let text = r#"{"action":"accept","content":{"decimal":1.23000,"exponent":2.5e-1},"_meta":{"huge":18446744073709551616,"beyond":1e400}}"#;
    let decoded =
        codec::decode::<v1::CreateElicitationResponse>(RawJson::parse(text).unwrap()).unwrap();
    let v1::ElicitationAction::Accept(action) = decoded.value.action else {
        panic!("accept action");
    };
    let values = action.content.unwrap();
    assert_eq!(values["decimal"], v1::ElicitationContentValue::Number(1.23));
    assert_eq!(
        values["exponent"],
        v1::ElicitationContentValue::Number(0.25)
    );
    assert_eq!(decoded.raw.text(), text);
    assert!(
        codec::decode::<v1::ElicitationContentValue>(RawJson::parse("1e400").unwrap()).is_err()
    );
}

#[test]
fn deep_values_are_rejected_before_unbounded_encode_recursion() {
    let mut value = serde_json::Value::Null;
    for _ in 0..=fpas_acp::raw::MAX_DEPTH {
        value = serde_json::Value::Array(vec![value]);
    }
    assert_eq!(kind(codec::encode(&value, 4096)), ErrorKind::DepthExceeded);
    let text = format!(
        "{}null{}",
        "[".repeat(fpas_acp::raw::MAX_DEPTH + 1),
        "]".repeat(fpas_acp::raw::MAX_DEPTH + 1)
    );
    assert_eq!(kind(RawJson::parse(&text)), ErrorKind::DepthExceeded);
}
