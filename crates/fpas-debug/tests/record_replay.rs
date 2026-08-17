//! JSONL reverse-execution and record/replay freeze.

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
    let (program, diagnostics) = fpas_parser::parse("program RecordReplay; begin end.");
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile record/replay fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn jsonl_record_replay_is_advertised_false_and_rejects_without_launch() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["reverse_execution"],
        false
    );
    assert_eq!(
        initialized[0]["body"]["capabilities"]["record_replay"],
        false
    );
    assert_eq!(
        initialized[0]["body"]["capabilities"]["recording_describe"],
        true
    );
    assert_eq!(
        initialized[0]["body"]["capabilities"]["recording_capture"],
        true
    );
    assert_eq!(
        initialized[0]["body"]["capabilities"]["recording_disk"],
        false
    );
    assert_eq!(
        initialized[0]["body"]["limits"]["recording_events"],
        fpas_vm::MAX_RECORDING_EVENTS
    );
    assert_eq!(initialized[0]["body"]["limits"]["recording_snapshots"], 0);
    assert_eq!(server.status(), ServerStatus::Initialized);

    for (id, command) in [(2, "step_back"), (3, "reverse_continue"), (4, "replay")] {
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
fn jsonl_record_replay_after_stop_does_not_resume() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(3, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");

    for (id, command) in [(4, "step_back"), (5, "reverse_continue"), (6, "replay")] {
        let rejected = server.handle_line(&request(id, command, json!({})));
        assert_eq!(
            rejected[0]["error"]["code"], "unsupported_capability",
            "{command}: {rejected:?}"
        );
        assert_eq!(rejected.len(), 1, "{command}: {rejected:?}");
    }
    let recorded = server.handle_line(&request(7, "record", json!({})));
    assert_eq!(recorded[0]["success"], true, "{recorded:?}");
    assert_eq!(recorded[0]["body"]["capturing"], true);
    assert_eq!(recorded[0]["body"]["truncated"], false);
    assert_eq!(
        recorded[0]["body"]["event_limit"],
        fpas_vm::MAX_RECORDING_EVENTS
    );
    assert_eq!(recorded.len(), 1, "{recorded:?}");
    assert_eq!(server.status(), ServerStatus::Stopped);

    let same_stack = server.handle_line(&request(8, "stack", json!({})));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}

#[test]
fn jsonl_recording_describe_names_portable_identity_without_recording() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let described = server.handle_line(&request(2, "recording.describe", json!({})));
    assert_eq!(described[0]["success"], true, "{described:?}");
    assert_eq!(described[0]["body"]["version"], 1);
    assert_eq!(
        described[0]["body"]["bytecode_version"],
        fpas_bytecode::BYTECODE_VERSION
    );
    assert_eq!(described[0]["body"]["program"], "recordreplay");
    assert_eq!(described[0]["body"]["sources"], json!(["<memory>"]));
    assert_eq!(described[0]["body"]["capturing"], false);
    assert_eq!(described[0]["body"]["truncated"], false);
    assert_eq!(described[0]["body"]["event_count"], 0);
    assert_eq!(
        described[0]["body"]["event_limit"],
        fpas_vm::MAX_RECORDING_EVENTS
    );
    assert_eq!(described[0]["body"]["events"], json!([]));
    assert_eq!(described.len(), 1, "{described:?}");
    assert_eq!(server.status(), ServerStatus::Initialized);

    let recorded = server.handle_line(&request(3, "record", json!({})));
    assert_eq!(recorded[0]["success"], true, "{recorded:?}");
    assert_eq!(recorded[0]["body"]["capturing"], true);
    assert_eq!(server.status(), ServerStatus::Initialized);

    let described = server.handle_line(&request(4, "recording.describe", json!({})));
    assert_eq!(described[0]["body"]["capturing"], true);
    assert_eq!(described[0]["body"]["truncated"], false);
    assert_eq!(described[0]["body"]["events"][0]["kind"], "stop");
    assert_eq!(described[0]["body"]["events"][0]["reason"], "entry");
}

#[test]
fn jsonl_record_captures_queued_input_without_replay() {
    let source = r#"program CaptureInput;
uses Std.Console;
begin
  WriteLn(ReadLn())
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile capture fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server");
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let _ = server.handle_line(&request(3, "record", json!({})));
    let queued = server.handle_line(&request(4, "io.input", json!({"text":"hello"})));
    assert_eq!(queued[0]["success"], true, "{queued:?}");
    let described = server.handle_line(&request(5, "recording.describe", json!({})));
    assert_eq!(described[0]["body"]["capturing"], true);
    let events = described[0]["body"]["events"].as_array().expect("events");
    assert!(
        events
            .iter()
            .any(|event| event["kind"] == "input" && event["text"] == "hello"),
        "{described:?}"
    );
    let rejected = server.handle_line(&request(6, "replay", json!({})));
    assert_eq!(rejected[0]["error"]["code"], "unsupported_capability");
    assert_eq!(server.status(), ServerStatus::Stopped);
}

#[test]
fn jsonl_recording_describe_after_stop_does_not_resume() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(3, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");

    let described = server.handle_line(&request(4, "recording.describe", json!({})));
    assert_eq!(described[0]["success"], true, "{described:?}");
    assert_eq!(described[0]["body"]["capturing"], false);
    assert_eq!(described.len(), 1, "{described:?}");
    assert_eq!(server.status(), ServerStatus::Stopped);

    let same_stack = server.handle_line(&request(5, "stack", json!({})));
    assert_eq!(same_stack[0]["body"]["frames"][0]["frame_id"], frame);
}
