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
fn jsonl_hot_reload_is_advertised_false_and_rejects_without_launch() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(initialized[0]["body"]["capabilities"]["hot_reload"], false);
    assert_eq!(
        initialized[0]["body"]["capabilities"]["record_replay"],
        false
    );
    assert_eq!(server.status(), ServerStatus::Initialized);

    for (id, command) in [(2, "reload"), (3, "image.replace")] {
        let rejected = server.handle_line(&request(id, command, json!({})));
        assert_eq!(
            rejected[0]["error"]["code"], "unsupported_capability",
            "{command}: {rejected:?}"
        );
        assert_eq!(rejected.len(), 1, "{command}: {rejected:?}");
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
        let rejected = server.handle_line(&request(id, command, json!({})));
        assert_eq!(
            rejected[0]["error"]["code"], "unsupported_capability",
            "{command}: {rejected:?}"
        );
        assert_eq!(rejected.len(), 1, "{command}: {rejected:?}");
    }
    assert_eq!(server.status(), ServerStatus::Stopped);

    let same_stack = server.handle_line(&request(8, "stack", json!({})));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}
