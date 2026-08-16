//! JSONL rejected instruction-pointer coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program InstructionChange;

uses Std.Console;

function Branch(Value: integer): integer;
begin
  mutable var Local: integer := Value + 10;
  WriteLn('effect');
  return Local
end;

begin
  WriteLn(Branch(1))
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile instruction-change fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn jsonl_instruction_set_is_advertised_false_and_rejects_atomically() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["instruction_set"],
        false
    );
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(3, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");
    let instruction = stack[0]["body"]["frames"][0]
        .get("instruction")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let rejected = server.handle_line(&request(
        4,
        "instruction.set",
        json!({"frame_id":frame,"instruction":instruction.saturating_add(1)}),
    ));
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(
        rejected[0]["error"]["code"],
        "instruction_change_unsupported"
    );
    assert_eq!(
        rejected.len(),
        1,
        "rejection must not resume or emit output"
    );

    let current = server.handle_line(&request(
        5,
        "instruction.set",
        json!({"frame_id":frame,"instruction":instruction}),
    ));
    assert_eq!(
        current[0]["error"]["code"],
        "instruction_change_unsupported"
    );

    let same_stack = server.handle_line(&request(6, "stack", json!({})));
    assert_eq!(
        same_stack[0]["body"]["frames"][0]["frame_id"], frame,
        "inspection generation must stay valid"
    );
}

#[test]
fn jsonl_instruction_set_rejects_stale_frames_before_the_decision() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let stack = server.handle_line(&request(3, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("entry frame");
    let _ = server.handle_line(&request(4, "step_into", json!({})));
    let _ = server.wait();
    let stale = server.handle_line(&request(
        5,
        "instruction.set",
        json!({"frame_id":frame,"instruction":0}),
    ));
    assert_eq!(stale[0]["error"]["code"], "unknown_frame");
}
