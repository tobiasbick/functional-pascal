//! JSONL per-task pause and resume holds.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TaskControl;

uses Std.Console, Std.Task;

function Work(): integer;
begin
  mutable var Value: integer := 40;
  Value := Value + 2;
  return Value
end;

begin
  var Pending: task := go Work();
  WriteLn(Wait(Pending))
end.
"#;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task-control fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("debug server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn wait_until_stopped(server: &mut JsonlServer) -> Vec<Value> {
    let records = server.wait();
    assert_eq!(server.status(), ServerStatus::Stopped, "{records:?}");
    records
}

fn stop_in_child(server: &mut JsonlServer) {
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let events = wait_until_stopped(server);
    assert!(events.iter().any(|record| {
        record["event"] == "stopped"
            && record["body"]["task_id"] == 1
            && record["body"]["all_tasks_stopped"] == true
    }));
}

fn child(tasks: &Value) -> &Value {
    tasks["body"]["tasks"]
        .as_array()
        .expect("tasks")
        .iter()
        .find(|task| task["task_id"] == 1)
        .expect("child")
}

#[test]
fn initialize_advertises_task_pause_without_non_stop() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(initialized[0]["body"]["capabilities"]["task_pause"], true);
    assert_eq!(initialized[0]["body"]["capabilities"]["non_stop"], false);
}

#[test]
fn pause_and_resume_require_a_current_task_id() {
    let mut server = server();
    stop_in_child(&mut server);
    let before = server.handle_line(&request(4, "tasks", json!({})));
    assert_eq!(child(&before[0])["paused"], false);

    let missing = server.handle_line(&request(5, "task.pause", json!({})));
    assert_eq!(missing[0]["success"], false, "{missing:?}");
    assert_eq!(missing[0]["error"]["code"], "invalid_request");

    let unknown = server.handle_line(&request(6, "task.pause", json!({"task_id":99})));
    assert_eq!(unknown[0]["success"], false, "{unknown:?}");
    assert_eq!(unknown[0]["error"]["code"], "unknown_task");
    let after = server.handle_line(&request(7, "tasks", json!({})));
    assert_eq!(after[0]["body"], before[0]["body"]);
}

#[test]
fn paused_child_is_catalogued_and_skipped_until_resume() {
    let mut server = server();
    stop_in_child(&mut server);

    let paused = server.handle_line(&request(4, "task.pause", json!({"task_id":1})));
    assert_eq!(paused[0]["success"], true, "{paused:?}");
    assert_eq!(paused[0]["body"]["task_id"], 1);
    assert_eq!(paused[0]["body"]["paused"], true);
    let catalog = server.handle_line(&request(5, "tasks", json!({})));
    assert_eq!(child(&catalog[0])["paused"], true);
    assert_eq!(server.status(), ServerStatus::Stopped);

    let held = server.handle_line(&request(6, "continue", json!({})));
    assert_eq!(held[0]["success"], true, "{held:?}");
    let blocked = wait_until_stopped(&mut server);
    assert!(
        blocked
            .iter()
            .any(|record| { record["event"] == "stopped" && record["body"]["reason"] == "pause" })
    );
    let still_held = server.handle_line(&request(7, "tasks", json!({})));
    assert_eq!(child(&still_held[0])["paused"], true);

    let resumed = server.handle_line(&request(8, "task.resume", json!({"task_id":1})));
    assert_eq!(resumed[0]["success"], true, "{resumed:?}");
    assert_eq!(resumed[0]["body"]["paused"], false);
    let _ = server.handle_line(&request(9, "continue", json!({})));
    let terminated = server.wait();
    assert_eq!(server.status(), ServerStatus::Terminated, "{terminated:?}");
    assert!(
        terminated
            .iter()
            .any(|event| event["event"] == "output" && event["body"]["text"] == "42\n")
    );
}
