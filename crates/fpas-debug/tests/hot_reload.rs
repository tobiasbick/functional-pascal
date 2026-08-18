//! JSONL live-image hot-reload freeze.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::{Value, json};

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse("program HotReload; begin end.");
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile hot-reload fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn jsonl_hot_reload_is_advertised_false_and_does_not_apply_without_launch() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(initialized[0]["body"]["capabilities"]["hot_reload"], false);
    assert_eq!(
        initialized[0]["body"]["capabilities"]["reload_classify"],
        true
    );
    assert_eq!(
        initialized[0]["body"]["capabilities"]["record_replay"],
        false
    );
    assert_eq!(server.status(), ServerStatus::Initialized);

    for (id, command) in [(2, "reload"), (3, "image.replace")] {
        let replaced = server.handle_line(&request(id, command, json!({})));
        assert_eq!(
            replaced[0]["body"]["class"], "unchanged",
            "{command}: {replaced:?}"
        );
        assert_eq!(replaced[0]["body"]["accepted"], true, "{command}");
        assert_eq!(replaced[0]["body"]["applied"], false, "{command}");
        assert_eq!(replaced.len(), 1, "{command}: {replaced:?}");
    }
    assert_eq!(server.status(), ServerStatus::Initialized);
}

#[test]
fn jsonl_hot_reload_after_stop_does_not_resume_or_replace_the_image() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(3, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");

    let _ = server.handle_line(&request(4, "record", json!({})));
    let described = server.handle_line(&request(5, "recording.describe", json!({})));
    assert_eq!(described[0]["body"]["replayable"], false);
    assert_eq!(described[0]["body"]["capturing"], true);

    for (id, command) in [(6, "reload"), (7, "image.replace")] {
        let replaced = server.handle_line(&request(id, command, json!({})));
        assert_eq!(
            replaced[0]["body"]["class"], "unchanged",
            "{command}: {replaced:?}"
        );
        assert_eq!(replaced[0]["body"]["applied"], false, "{command}");
        assert_eq!(replaced.len(), 1, "{command}: {replaced:?}");
    }
    assert_eq!(server.status(), ServerStatus::Stopped);

    let same_stack = server.handle_line(&request(8, "stack", json!({})));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}

#[test]
fn jsonl_reload_classify_names_classes_without_replacing_the_image() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let classified = server.handle_line(&request(2, "reload.classify", json!({})));
    assert_eq!(classified[0]["body"]["class"], "unchanged");
    assert_eq!(classified[0]["body"]["accepted"], true);
    assert_eq!(classified[0]["body"]["applied"], false);
    assert_eq!(
        classified[0]["body"]["accepted_classes"],
        json!(["unchanged", "inactive_function_body"])
    );
    assert!(
        classified[0]["body"]["rejected_classes"]
            .as_array()
            .is_some_and(|classes| classes.iter().any(|class| class == "active_function_body")),
        "{classified:?}"
    );

    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(4, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");
    let after_stop = server.handle_line(&request(5, "reload.classify", json!({})));
    assert_eq!(after_stop[0]["body"]["class"], "unchanged");
    assert_eq!(after_stop[0]["body"]["applied"], false);
    assert_eq!(server.status(), ServerStatus::Stopped);
    let same_stack = server.handle_line(&request(6, "stack", json!({})));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}

#[test]
fn jsonl_incompatible_replace_is_rejected_before_the_image_changes() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(3, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");

    let (program, diagnostics) = fpas_parser::parse(
        "program IncompatibleReload;\nfunction Extra(): integer;\nbegin\n  return 1\nend;\nbegin\nend.",
    );
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let candidate = fpas_compiler::compile(&program).expect("compile incompatible fixture");
    let error = server
        .replace_live_image(&candidate)
        .expect_err("incompatible replace");
    assert_eq!(error.kind, fpas_vm::DebugErrorKind::LiveImageIncompatible);
    assert_eq!(server.status(), ServerStatus::Stopped);
    let same_stack = server.handle_line(&request(4, "stack", json!({})));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}
