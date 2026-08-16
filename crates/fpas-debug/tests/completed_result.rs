//! JSONL retained completed-task result replacement coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program CompletedTaskResult;

uses Std.Console, Std.Task;

function Work(): integer;
begin
  return 7
end;

begin
  var Pending: task := go Work();
  WriteLn(Wait(Pending))
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile completed-result fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn jsonl_replaces_completed_retained_result_before_wait_consumes_it() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["task_result_replacement"],
        true
    );
    let breakpoint = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":7}),
    ));
    assert_eq!(breakpoint[0]["body"]["verified"], true, "{breakpoint:?}");
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let stopped = server.wait();
    assert!(
        stopped
            .iter()
            .any(|record| { record["event"] == "stopped" && record["body"]["task_id"] == 1 }),
        "{stopped:?}"
    );
    let child_stack = server.handle_line(&request(4, "stack", json!({"task_id":1})));
    let child_frame = child_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("child frame");
    let completed = server.handle_line(&request(
        5,
        "frame.return",
        json!({"frame_id":child_frame,"expression":"7"}),
    ));
    assert_eq!(completed[0]["success"], true, "{completed:?}");
    assert_eq!(completed[0]["body"]["task_id"], 1);

    let root_stack = server.handle_line(&request(6, "stack", json!({"task_id":0})));
    let root_frame = root_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("root frame");
    let mismatch = server.handle_line(&request(
        7,
        "task.result.replace",
        json!({"task_id":1,"frame_id":root_frame,"expression":"'wrong'"}),
    ));
    assert_eq!(
        mismatch[0]["error"]["code"], "task_result_replacement_type",
        "{mismatch:?}"
    );

    let replaced = server.handle_line(&request(
        8,
        "task.result.replace",
        json!({"task_id":1,"frame_id":root_frame,"expression":"9"}),
    ));
    assert_eq!(replaced[0]["success"], true, "{replaced:?}");
    assert_eq!(replaced[0]["body"]["task_id"], 1);
    assert_eq!(replaced[0]["body"]["result"], "9");
    assert_eq!(replaced[0]["body"]["type_name"], "integer");

    let refreshed = server.handle_line(&request(9, "stack", json!({"task_id":0})));
    let refreshed_frame = refreshed[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("refreshed root frame");
    let replaced_again = server.handle_line(&request(
        10,
        "task.result.replace",
        json!({"task_id":1,"frame_id":refreshed_frame,"expression":"10"}),
    ));
    assert_eq!(replaced_again[0]["body"]["result"], "10");

    let _ = server.handle_line(&request(11, "continue", json!({})));
    let completed = server.wait();
    assert!(
        completed
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "10\n" })
    );
    assert!(
        completed
            .iter()
            .any(|record| record["event"] == "terminated")
    );
}

#[test]
fn jsonl_completed_result_request_validates_task_and_expression_fields() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));

    let missing = server.handle_line(&request(3, "task.result.replace", json!({})));
    assert_eq!(missing[0]["error"]["code"], "invalid_request");
    let invalid = server.handle_line(&request(
        4,
        "task.result.replace",
        json!({"task_id":"child","expression":9}),
    ));
    assert_eq!(invalid[0]["error"]["code"], "invalid_request");
}
