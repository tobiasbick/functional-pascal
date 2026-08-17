//! JSONL data-breakpoint freeze until durable location identities exist.

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
    let (program, diagnostics) = fpas_parser::parse("program DataBreakpoints; begin end.");
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile data-breakpoint fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn jsonl_data_breakpoints_are_advertised_false_and_reject_without_launch() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["data_breakpoints"],
        false
    );
    assert_eq!(server.status(), ServerStatus::Initialized);

    for (id, command) in [(2, "data_breakpoint.set"), (3, "data_breakpoints.replace")] {
        let rejected = server.handle_line(&request(id, command, json!({})));
        assert_eq!(rejected[0]["success"], false, "{command}: {rejected:?}");
        assert_eq!(rejected[0]["error"]["code"], "unsupported_capability");
        assert!(
            rejected[0]["error"]["help"]
                .as_str()
                .is_some_and(|help| help.contains("expire")),
            "{command}: {rejected:?}"
        );
        assert_eq!(
            rejected.len(),
            1,
            "{command}: rejection must not launch or emit events"
        );
    }
    assert_eq!(server.status(), ServerStatus::Initialized);

    let launched = server.handle_line(&request(4, "launch", json!({"stop_on_entry":true})));
    assert!(
        launched.iter().any(|record| record["event"] == "stopped"),
        "{launched:?}"
    );
    assert_eq!(server.status(), ServerStatus::Stopped);
}

#[test]
fn jsonl_data_breakpoint_reject_after_stop_does_not_resume() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(3, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");

    for (id, command) in [(4, "data_breakpoint.set"), (5, "data_breakpoints.replace")] {
        let rejected = server.handle_line(&request(id, command, json!({})));
        assert_eq!(
            rejected[0]["error"]["code"], "unsupported_capability",
            "{command}: {rejected:?}"
        );
        assert_eq!(rejected.len(), 1, "{command}: {rejected:?}");
    }
    assert_eq!(server.status(), ServerStatus::Stopped);

    let same_stack = server.handle_line(&request(6, "stack", json!({})));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}
