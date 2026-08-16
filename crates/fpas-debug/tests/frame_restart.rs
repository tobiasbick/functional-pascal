//! JSONL selected live-frame restart coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program FrameRestart;

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
    let executable = fpas_compiler::compile(&program).expect("compile restart fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn jsonl_restart_reenters_selected_frame_without_running_it() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["frame_restart"],
        true
    );
    let breakpoint = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":9}),
    ));
    assert_eq!(breakpoint[0]["body"]["verified"], true, "{breakpoint:?}");
    let breakpoint_id = breakpoint[0]["body"]["breakpoint_id"]
        .as_u64()
        .expect("breakpoint ID");
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let stopped = server.wait();
    assert!(
        stopped
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "effect\n" })
    );
    let cleared = server.handle_line(&request(
        4,
        "breakpoint.clear",
        json!({"breakpoint_id":breakpoint_id}),
    ));
    assert_eq!(cleared[0]["success"], true, "{cleared:?}");
    let stack = server.handle_line(&request(5, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("branch frame");

    let restarted = server.handle_line(&request(6, "frame.restart", json!({"frame_id":frame})));
    assert_eq!(restarted[0]["success"], true, "{restarted:?}");
    assert_eq!(restarted[0]["body"]["task_id"], 0);
    assert_eq!(restarted[0]["body"]["frame"]["name"], "branch");
    assert_eq!(restarted[0]["body"]["discarded_frames"], 0);
    assert_eq!(
        restarted.len(),
        1,
        "restart must not execute or emit output"
    );

    let stale = server.handle_line(&request(7, "frame.restart", json!({"frame_id":frame})));
    assert_eq!(stale[0]["error"]["code"], "unknown_frame");
    let _ = server.handle_line(&request(8, "continue", json!({})));
    let completed = server.wait();
    let output = completed
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "effect\n11\n");
}

#[test]
fn jsonl_restart_requires_a_current_frame() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let missing = server.handle_line(&request(3, "frame.restart", json!({})));
    assert_eq!(missing[0]["error"]["code"], "invalid_request");
}
