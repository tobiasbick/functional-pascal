//! Independent wire fixtures protect nominal families, defaults, patches and raw data.

use fpas_acp::{
    RawJson,
    binding::{Catalog, Shape, Value, catalog},
    schema::Version,
};

fn field<'a>(catalog: &Catalog, key: &str, value: &'a Value, name: &str) -> &'a Value {
    let Shape::Record(fields) = &catalog.declarations[key].shape else {
        panic!("record declaration");
    };
    let Value::Record { fields: values, .. } = value else {
        panic!("record value");
    };
    &values[fields
        .iter()
        .position(|field| field.name == name)
        .expect("field")]
}

#[test]
fn stable_and_v2_wire_methods_choose_their_own_nominal_payloads() {
    for (version, root, method, text, variant) in [
        (
            Version::V1,
            "ClientRequest",
            "initialize",
            r#"{"protocolVersion":1,"clientInfo":false,"_meta":{"n":9007199254740993,"big":1e400}}"#,
            "InitializeRequest",
        ),
        (
            Version::V1,
            "ClientRequest",
            "session/prompt",
            r#"{"sessionId":"s","prompt":[{"type":"text","text":"hi"},{"type":"image","data":"AA==","mimeType":"image/png"}]}"#,
            "PromptRequest",
        ),
        (
            Version::V1,
            "AgentRequest",
            "fs/read_text_file",
            r#"{"sessionId":"s","path":"/tmp/example","line":1,"limit":3}"#,
            "ReadTextFileRequest",
        ),
        (
            Version::V1,
            "ClientResponse",
            "terminal/wait_for_exit",
            r#"{"exitCode":4294967295,"signal":null}"#,
            "WaitForTerminalExitResponse",
        ),
        (
            Version::V1,
            "ProtocolLevelNotification",
            "$/cancel_request",
            r#"{"requestId":-9223372036854775808}"#,
            "CancelRequestNotification",
        ),
        (
            Version::V2,
            "ClientRequest",
            "initialize",
            r#"{"protocolVersion":2,"info":{"name":"fixture","version":"1"},"capabilities":{}}"#,
            "InitializeRequest",
        ),
        (
            Version::V2,
            "ClientRequest",
            "session/prompt",
            r#"{"sessionId":"s","prompt":[{"type":"text","text":"hi"}]}"#,
            "PromptRequest",
        ),
        (
            Version::V2,
            "AgentNotification",
            "session/update",
            r#"{"sessionId":"s","update":{"sessionUpdate":"tool_call_update","toolCallId":"tool","title":null,"content":[]}}"#,
            "UpdateSessionNotification",
        ),
    ] {
        let catalog = catalog(version).unwrap();
        let raw = RawJson::parse(text).unwrap();
        let decoded = catalog
            .decode_message(root, method, raw)
            .unwrap_or_else(|error| panic!("{version:?} {method}: {error}"));
        let Value::Enum {
            name,
            variant: actual,
            ..
        } = &decoded.payload
        else {
            panic!("typed root");
        };
        assert_eq!(actual, variant);
        assert_eq!(
            name,
            &format!(
                "Std.Acp.{}.{root}",
                if version == Version::V1 {
                    "Types"
                } else {
                    "V2"
                }
            )
        );
        assert_eq!(decoded.raw.text(), text);
        catalog
            .encode_message(root, method, &decoded.payload)
            .unwrap_or_else(|error| panic!("encode {version:?} {method}: {error}"));
    }
}

#[test]
fn patch_presence_and_arbitrary_meta_are_observable_without_float_conversion() {
    for version in [Version::V1, Version::V2] {
        let catalog = catalog(version).unwrap();
        for (text, expected) in [
            (r#"{"messageId":"m"}"#, Value::Missing),
            (r#"{"messageId":"m","content":null}"#, Value::Null),
            (
                r#"{"messageId":"m","content":[]}"#,
                Value::Replacement(Box::new(Value::Array(vec![]))),
            ),
        ] {
            let decoded = catalog
                .decode("SessionMessage", &RawJson::parse(text).unwrap())
                .unwrap();
            assert_eq!(
                field(catalog, "SessionMessage", &decoded, "Content"),
                &expected
            );
        }
        let raw = r#"{"messageId":"m","_meta":{"max":18446744073709551615,"beyond":18446744073709551616,"spelling":1e400}}"#;
        let decoded = catalog
            .decode("SessionMessage", &RawJson::parse(raw).unwrap())
            .unwrap();
        let Value::Replacement(meta) = field(catalog, "SessionMessage", &decoded, "Meta") else {
            panic!("meta patch");
        };
        let Value::Raw(meta) = &**meta else {
            panic!("raw metadata");
        };
        assert_eq!(
            meta.text(),
            r#"{"max":18446744073709551615,"beyond":18446744073709551616,"spelling":1e400}"#
        );
    }
}

#[test]
fn unknown_content_is_data_and_malformed_known_items_follow_sdk_skip_rules() {
    let catalog = catalog(Version::V1).unwrap();
    let text = r#"{"messageId":"m","content":[{"type":"_future","exact":18446744073709551616},{"type":"text","text":false},{"type":"text","text":"retained"}]}"#;
    let decoded = catalog
        .decode("SessionMessage", &RawJson::parse(text).unwrap())
        .unwrap();
    let Value::Replacement(content) = field(catalog, "SessionMessage", &decoded, "Content") else {
        panic!("content patch");
    };
    let Value::Array(prompt) = &**content else {
        panic!("content");
    };
    assert_eq!(prompt.len(), 2);
    assert!(
        matches!(&prompt[0], Value::Enum { variant, fields, .. } if variant == "Unknown" && matches!(&fields[1], Value::Raw(raw) if raw.text() == r#"{"type":"_future","exact":18446744073709551616}"#))
    );
    let output = catalog.encode("SessionMessage", &decoded).unwrap();
    assert!(output.text().contains("18446744073709551616"));
    assert!(!output.text().contains("false"));
}

#[test]
fn mandatory_fields_bounds_and_method_family_mismatches_are_rejected() {
    let catalog = catalog(Version::V1).unwrap();
    for text in [
        r#"{}"#,
        r#"{"sessionId":4,"prompt":[]}"#,
        r#"{"sessionId":"s","prompt":false}"#,
    ] {
        assert!(
            catalog
                .decode_message(
                    "ClientRequest",
                    "session/prompt",
                    RawJson::parse(text).unwrap()
                )
                .is_err()
        );
    }
    let status = catalog
        .decode(
            "TerminalExitStatus",
            &RawJson::parse(r#"{"exitCode":4294967296}"#).unwrap(),
        )
        .unwrap();
    assert_eq!(
        field(catalog, "TerminalExitStatus", &status, "ExitCode"),
        &Value::None
    );
    assert!(
        catalog
            .decode_message(
                "ClientRequest",
                "initialize",
                RawJson::parse(r#"{"protocolVersion":65536}"#).unwrap()
            )
            .is_err()
    );
    let request = catalog
        .decode_message(
            "ClientRequest",
            "session/prompt",
            RawJson::parse(r#"{"sessionId":"s","prompt":[]}"#).unwrap(),
        )
        .unwrap();
    assert!(
        catalog
            .encode_message("AgentResponse", "session/prompt", &request.payload)
            .is_err()
    );
    assert!(
        catalog
            .encode_message("ClientRequest", "initialize", &request.payload)
            .is_err()
    );
}

#[test]
fn extension_methods_and_error_data_preserve_the_exact_wire_payload() {
    let catalog = catalog(Version::V1).unwrap();
    let text = r#"{"custom":[-9223372036854775808,18446744073709551616,1e400]}"#;
    let decoded = catalog
        .decode_message("ClientRequest", "__custom", RawJson::parse(text).unwrap())
        .unwrap();
    assert_eq!(
        catalog
            .encode_message("ClientRequest", "__custom", &decoded.payload)
            .unwrap()
            .text(),
        text
    );
    let error = catalog
        .decode(
            "Error",
            &RawJson::parse(
                r#"{"code":-32603,"message":"peer","data":{"n":18446744073709551616}}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        field(catalog, "Error", &error, "Code"),
        &Value::Integer(-32603)
    );
    assert!(
        matches!(field(catalog, "Error", &error, "Data"), Value::Some(value) if matches!(&**value, Value::Raw(raw) if raw.text() == r#"{"n":18446744073709551616}"#))
    );
}

#[test]
fn outbound_values_reject_nonfinite_reals_and_out_of_domain_integers() {
    let catalog = catalog(Version::V1).unwrap();
    for amount in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let cost = Value::Record {
            name: catalog.declarations["Cost"].name.clone(),
            fields: vec![
                Value::Real(amount),
                Value::String("USD".into()),
                Value::None,
            ],
        };
        assert!(catalog.encode("Cost", &cost).is_err());
    }
    let status = Value::Record {
        name: catalog.declarations["TerminalExitStatus"].name.clone(),
        fields: vec![
            Value::Some(Box::new(Value::Integer(4_294_967_296))),
            Value::None,
            Value::None,
        ],
    };
    assert!(catalog.encode("TerminalExitStatus", &status).is_err());
}

#[test]
fn an_omitted_select_discriminator_is_a_known_sdk_default() {
    let catalog = catalog(Version::V1).unwrap();
    for text in [
        r#"{"sessionId":"s","configId":"model","value":"luna"}"#,
        r#"{"sessionId":"s","configId":"model","type":"select","value":"luna"}"#,
    ] {
        let decoded = catalog
            .decode(
                "SetSessionConfigOptionRequest",
                &RawJson::parse(text).unwrap(),
            )
            .unwrap();
        assert!(
            matches!(field(catalog, "SetSessionConfigOptionRequest", &decoded, "Kind"), Value::Enum { variant, .. } if variant == "Select")
        );
        catalog
            .encode("SetSessionConfigOptionRequest", &decoded)
            .unwrap();
    }
}
