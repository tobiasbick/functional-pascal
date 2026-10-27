#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! Independent content, configuration, elicitation and update examples.

#[path = "methods/v2_updates.rs"]
mod v2_updates;

use fpas_acp_codec::binding::CatalogCodec as _;
use fpas_acp_codec::{
    RawJson,
    binding::{Shape, Value, catalog},
    schema::Version,
};

fn check(version: Version, key: &str, text: &str, expected: Option<&str>) {
    let catalog = catalog(version).unwrap();
    let value = catalog
        .decode(key, &RawJson::parse(text).unwrap())
        .unwrap_or_else(|error| panic!("{version:?} {key} {text}: {error}"));
    if let Some(expected) = expected {
        let Value::Enum { variant, .. } = &value else {
            panic!("expected enum");
        };
        assert_eq!(variant, expected, "{version:?} {key} {text}");
    }
    catalog
        .encode(key, &value)
        .unwrap_or_else(|error| panic!("encode {version:?} {key} {text}: {error}"));
}

#[test]
fn content_mcp_tool_and_form_variants_keep_their_tagged_payloads() {
    for version in [Version::V1, Version::V2] {
        for (key, variant, text) in [
            (
                "ContentBlock",
                "Text",
                r#"{"type":"text","text":"hello","annotations":{"audience":["user","assistant"],"priority":0.5,"lastModified":"now"}}"#,
            ),
            (
                "ContentBlock",
                "Image",
                r#"{"type":"image","data":"AA==","mimeType":"image/png","uri":"file:///x"}"#,
            ),
            (
                "ContentBlock",
                "Audio",
                r#"{"type":"audio","data":"AA==","mimeType":"audio/wav"}"#,
            ),
            (
                "ContentBlock",
                "ResourceLink",
                r#"{"type":"resource_link","uri":"file:///x","name":"x","size":9007199254740993}"#,
            ),
            (
                "ContentBlock",
                "Resource",
                r#"{"type":"resource","resource":{"uri":"file:///x","text":"data","mimeType":"text/plain"}}"#,
            ),
            (
                "ContentBlock",
                "Resource",
                r#"{"type":"resource","resource":{"uri":"file:///x","blob":"AA==","mimeType":"application/octet-stream"}}"#,
            ),
            (
                "ContentBlock",
                "Unknown",
                r#"{"type":"_future","raw":[18446744073709551616,1e400]}"#,
            ),
            (
                "ToolCallContent",
                "Content",
                r#"{"type":"content","content":{"type":"text","text":"hello"}}"#,
            ),
            (
                "ToolCallContent",
                "Diff",
                if version == Version::V1 {
                    r#"{"type":"diff","path":"/x","oldText":null,"newText":"new"}"#
                } else {
                    r#"{"type":"diff","changes":[{"type":"add","path":"/x"}],"patch":{"format":"git_patch","text":"patch"}}"#
                },
            ),
            (
                "ToolCallContent",
                "Terminal",
                r#"{"type":"terminal","terminalId":"t"}"#,
            ),
            (
                "RequestPermissionOutcome",
                "Cancelled",
                r#"{"outcome":"cancelled"}"#,
            ),
            (
                "RequestPermissionOutcome",
                "Selected",
                r#"{"outcome":"selected","optionId":"once"}"#,
            ),
            (
                "PlanUpdateContent",
                "Items",
                r#"{"type":"items","planId":"p","entries":[{"content":"Do it","priority":"high","status":"pending"}]}"#,
            ),
            (
                "PlanUpdateContent",
                "File",
                r#"{"type":"file","planId":"p","uri":"file:///plan"}"#,
            ),
            (
                "PlanUpdateContent",
                "Markdown",
                r##"{"type":"markdown","planId":"p","content":"# Plan"}"##,
            ),
            ("StateUpdate", "Running", r#"{"state":"running"}"#),
            (
                "StateUpdate",
                "Idle",
                r#"{"state":"idle","stopReason":"end_turn"}"#,
            ),
            (
                "StateUpdate",
                "RequiresAction",
                r#"{"state":"requires_action"}"#,
            ),
        ] {
            check(version, key, text, Some(variant));
        }
        for text in [
            r#"{"message":"Fill","mode":"form","sessionId":"s","requestedSchema":{"type":"object","properties":{"s":{"type":"string"},"n":{"type":"number"},"i":{"type":"integer"},"b":{"type":"boolean"}}}}"#,
            r#"{"message":"Open","mode":"url","requestId":-9223372036854775808,"url":"https://example.test","elicitationId":"e"}"#,
        ] {
            check(version, "CreateElicitationRequest", text, None);
        }
        for text in [
            r#"{"action":"accept","content":{"s":"value","n":2.5,"i":2,"b":true,"a":["x"]}}"#,
            r#"{"action":"decline"}"#,
            r#"{"action":"cancel"}"#,
        ] {
            check(version, "CreateElicitationResponse", text, None);
        }
        for kind in [
            "allow_once",
            "allow_always",
            "reject_once",
            "reject_always",
            "_future",
        ] {
            check(
                version,
                "PermissionOptionKind",
                &format!("\"{kind}\""),
                None,
            );
        }
    }
}

#[test]
fn every_v1_session_update_variant_has_an_independent_example() {
    let fixtures = [
        r#"{"sessionUpdate":"user_message_chunk","content":{"type":"text","text":"hi"}}"#,
        r#"{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"hi"}}"#,
        r#"{"sessionUpdate":"agent_thought_chunk","content":{"type":"text","text":"hi"}}"#,
        r#"{"sessionUpdate":"tool_call","toolCallId":"t","title":"Read","kind":"read","status":"pending","content":[],"locations":[{"path":"/x","line":1}],"rawInput":{"n":18446744073709551616}}"#,
        r#"{"sessionUpdate":"tool_call_update","toolCallId":"t","title":"Updated","content":[]}"#,
        r#"{"sessionUpdate":"plan","entries":[{"content":"Work","priority":"medium","status":"completed"}]}"#,
        r#"{"sessionUpdate":"available_commands_update","availableCommands":[{"name":"help","description":"Help","input":{"hint":"question"}}]}"#,
        r#"{"sessionUpdate":"current_mode_update","currentModeId":"ask"}"#,
        r#"{"sessionUpdate":"config_option_update","configOptions":[{"id":"model","name":"Model","type":"select","currentValue":"luna","options":[{"group":"g","name":"Group","options":[{"value":"luna","name":"Luna"}]}]}]}"#,
        r#"{"sessionUpdate":"session_info_update","title":null,"updatedAt":"now"}"#,
        r#"{"sessionUpdate":"usage_update","used":9007199254740993,"size":18446744073709551615,"cost":{"amount":0.1,"currency":"USD"}}"#,
        r#"{"sessionUpdate":"notice","severity":"info","title":"Notice","description":"Details"}"#,
        r#"{"sessionUpdate":"compaction_update","compactionId":"c","status":"completed","summary":[],"error":null}"#,
        r#"{"sessionUpdate":"compaction_summary_chunk","compactionId":"c","content":{"type":"text","text":"summary"}}"#,
        r#"{"sessionUpdate":"plan_update","plan":{"type":"items","planId":"p","entries":[]}}"#,
        r#"{"sessionUpdate":"plan_removed","planId":"p"}"#,
        r#"{"sessionUpdate":"subagent_update","sessionId":"child","state":{"state":"idle"},"title":null}"#,
        r#"{"sessionUpdate":"session_message","messageId":"m","content":null}"#,
        r#"{"sessionUpdate":"session_message_chunk","messageId":"m","content":{"type":"text","text":"hi"}}"#,
    ];
    let catalog = catalog(Version::V1).unwrap();
    let mut covered = std::collections::BTreeSet::new();
    for text in fixtures {
        let value = catalog
            .decode("SessionUpdate", &RawJson::parse(text).unwrap())
            .unwrap_or_else(|error| panic!("{text}: {error}"));
        let Value::Enum { variant, .. } = &value else {
            panic!("update enum");
        };
        assert_ne!(variant, "Unknown", "{text}");
        covered.insert(variant.clone());
        catalog
            .encode("SessionUpdate", &value)
            .unwrap_or_else(|error| panic!("encode {text}: {error}"));
    }
    let Shape::Enum { variants, .. } = &catalog.declarations["SessionUpdate"].shape else {
        panic!("update declaration");
    };
    assert_eq!(
        covered,
        variants
            .iter()
            .map(|variant| variant.name.clone())
            .collect()
    );
}

#[test]
fn flattened_future_configuration_and_mcp_error_data_remain_lossless() {
    for version in [Version::V1, Version::V2] {
        let catalog = catalog(version).unwrap();
        let text = r#"{"sessionId":"s","configId":"x","type":"_future","value":{"n":18446744073709551616},"extension":1e400,"_meta":{"n":9007199254740993}}"#;
        let decoded = catalog
            .decode_message(
                "ClientRequest",
                "session/set_config_option",
                RawJson::parse(text).unwrap(),
            )
            .unwrap();
        let output = catalog
            .encode_message(
                "ClientRequest",
                "session/set_config_option",
                &decoded.payload,
            )
            .unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(output.text()).unwrap(),
            serde_json::from_str::<serde_json::Value>(text).unwrap()
        );
        for text in [
            r#"{"code":-1,"message":"error"}"#,
            r#"{"code":-1,"message":"error","data":null,"extra":1e400}"#,
            r#"{"code":-1,"message":"error","data":{"n":18446744073709551616}}"#,
        ] {
            let decoded = catalog
                .decode("McpError", &RawJson::parse(text).unwrap())
                .unwrap();
            let output = catalog.encode("McpError", &decoded).unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(output.text()).unwrap(),
                serde_json::from_str::<serde_json::Value>(text).unwrap()
            );
        }
    }
}

#[test]
fn v1_agent_authentication_is_typed_with_or_without_its_default_tag() {
    for text in [
        r#"{"id":"login","name":"Login"}"#,
        r#"{"id":"login","name":"Login","type":"agent"}"#,
    ] {
        check(Version::V1, "AuthMethod", text, Some("Agent"));
    }
}

#[test]
fn every_v2_session_update_variant_has_an_independent_example() {
    let catalog = catalog(Version::V2).unwrap();
    let mut covered = std::collections::BTreeSet::new();
    for text in v2_updates::UPDATES {
        let value = catalog
            .decode("SessionUpdate", &RawJson::parse(text).unwrap())
            .unwrap_or_else(|error| panic!("{text}: {error}"));
        let Value::Enum { variant, .. } = &value else {
            panic!("update enum");
        };
        assert_ne!(variant, "Unknown", "{text}");
        covered.insert(variant.clone());
        catalog
            .encode("SessionUpdate", &value)
            .unwrap_or_else(|error| panic!("encode {text}: {error}"));
    }
    let Shape::Enum { variants, .. } = &catalog.declarations["SessionUpdate"].shape else {
        panic!("update declaration");
    };
    assert_eq!(
        covered,
        variants
            .iter()
            .map(|variant| variant.name.clone())
            .collect()
    );
}
