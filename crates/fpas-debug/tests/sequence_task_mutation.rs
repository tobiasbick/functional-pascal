//! Sequence mutation ownership coverage for selected stopped child tasks.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TaskSequenceMutation;

uses Std.Console, Std.Task;

function Work(): integer;
begin
  mutable var Values: array of integer := [1, 3];
  var Marker: integer := Values[0];
  return Values[0] + Values[1] + Values[2]
end;

begin
  var Pending: task := go Work();
  WriteLn(Wait(Pending))
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task sequence fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("debug server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn sequence_mutation_remains_bound_to_the_selected_child_task() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let breakpoint = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    assert_eq!(breakpoint[0]["body"]["verified"], true, "{breakpoint:?}");
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let stopped = server.wait();
    assert!(
        stopped
            .iter()
            .any(|record| { record["event"] == "stopped" && record["body"]["task_id"] == 1 })
    );

    let child_stack = server.handle_line(&request(4, "stack", json!({"task_id":1})));
    let child_frame = child_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("child frame");
    let main_stack = server.handle_line(&request(5, "stack", json!({"task_id":0})));
    let main_frame = main_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("main frame");

    let inserted = server.handle_line(&request(
        6,
        "array.insert",
        json!({"frame_id":child_frame,"target":"Values","index":"1","expression":"2"}),
    ));
    assert_eq!(inserted[0]["body"]["result"], "[3 items]", "{inserted:?}");
    let expired_main = server.handle_line(&request(7, "scopes", json!({"frame_id":main_frame})));
    assert_eq!(expired_main[0]["error"]["code"], "unknown_frame");

    let _ = server.handle_line(&request(8, "continue", json!({})));
    let terminated = server.wait();
    assert!(
        terminated
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "6\n" })
    );
}
