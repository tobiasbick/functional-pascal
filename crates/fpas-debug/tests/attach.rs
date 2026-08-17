//! JSONL launch-owned attach freeze and native-inspection rejection.

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
    let (program, diagnostics) = fpas_parser::parse("program Attach; begin end.");
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile attach fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn jsonl_attach_is_advertised_false_and_rejects_without_launch() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(initialized[0]["body"]["capabilities"]["attach"], false);
    assert_eq!(server.status(), ServerStatus::Initialized);

    let rejected = server.handle_line(&request(2, "attach", json!({})));
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected[0]["error"]["code"], "unsupported_capability");
    assert_eq!(
        rejected.len(),
        1,
        "rejection must not launch or emit events"
    );
    assert_eq!(server.status(), ServerStatus::Initialized);

    let launched = server.handle_line(&request(3, "launch", json!({"stop_on_entry":true})));
    assert!(
        launched.iter().any(|record| record["event"] == "stopped"),
        "{launched:?}"
    );
    assert_eq!(server.status(), ServerStatus::Stopped);
}

#[test]
fn jsonl_attach_after_stop_does_not_resume() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(3, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");

    let rejected = server.handle_line(&request(4, "attach", json!({})));
    assert_eq!(rejected[0]["error"]["code"], "unsupported_capability");
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(server.status(), ServerStatus::Stopped);

    let same_stack = server.handle_line(&request(5, "stack", json!({})));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}

#[test]
fn jsonl_native_inspection_commands_are_unsupported() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    for (id, command) in [(3, "disassemble"), (4, "registers"), (5, "readMemory")] {
        let rejected = server.handle_line(&request(id, command, json!({})));
        assert_eq!(
            rejected[0]["error"]["code"], "unsupported_capability",
            "{command}: {rejected:?}"
        );
        assert_eq!(rejected.len(), 1, "{command}: {rejected:?}");
    }
    assert_eq!(server.status(), ServerStatus::Stopped);
}

#[test]
fn jsonl_server_constructs_a_launch_owned_session_at_entry() {
    let mut server = server();
    assert_eq!(server.status(), ServerStatus::Created);
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(server.status(), ServerStatus::Initialized);
    let launched = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    assert!(
        launched.iter().any(|record| record["event"] == "stopped"),
        "{launched:?}"
    );
    assert_eq!(server.status(), ServerStatus::Stopped);
}
