//! JSONL all-stop catalog identity and stopped-event coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TaskQuiescence;

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
    let executable = fpas_compiler::compile(&program).expect("compile quiescence fixture");
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

fn stopped_events(records: &[Value]) -> Vec<&Value> {
    records
        .iter()
        .filter(|record| record["event"] == "stopped")
        .collect()
}

#[test]
fn initialize_advertises_all_stop_task_threads() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(initialized[0]["body"]["capabilities"]["task_threads"], true);
    assert_eq!(initialized[0]["body"]["capabilities"]["non_stop"], false);
    assert_eq!(initialized[0]["body"]["capabilities"]["pause"], true);
    assert_eq!(initialized[0]["body"]["capabilities"]["continue"], true);
}

#[test]
fn stopped_catalog_is_an_all_stop_snapshot() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let events = wait_until_stopped(&mut server);
    let stopped = stopped_events(&events);
    assert_eq!(stopped.len(), 1, "{events:?}");
    assert_eq!(stopped[0]["body"]["task_id"], 1);
    assert_eq!(stopped[0]["body"]["all_tasks_stopped"], true);

    let first = server.handle_line(&request(4, "tasks", json!({})));
    let second = server.handle_line(&request(5, "tasks", json!({})));
    assert_eq!(first[0]["body"], second[0]["body"]);
    assert_eq!(first[0]["body"]["total"], 2);
    assert_eq!(first[0]["body"]["tasks"][0]["task_id"], 0);
    assert_ne!(first[0]["body"]["tasks"][0]["state"], "completed");
    assert_ne!(first[0]["body"]["tasks"][0]["state"], "cancelled");
    assert_eq!(first[0]["body"]["tasks"][0]["inspectable"], true);
    assert_eq!(first[0]["body"]["tasks"][1]["task_id"], 1);
    assert_eq!(first[0]["body"]["tasks"][1]["state"], "runnable");
    assert_eq!(first[0]["body"]["tasks"][1]["inspectable"], true);

    let step = server.handle_line(&request(6, "step_into", json!({"task_id":1})));
    assert_eq!(step[0]["success"], true, "{step:?}");
    let stepped = wait_until_stopped(&mut server);
    assert!(stopped_events(&stepped).iter().any(|event| {
        event["body"]["reason"] == "step"
            && event["body"]["task_id"] == 1
            && event["body"]["all_tasks_stopped"] == true
    }));
}

#[test]
fn continue_ignores_task_id_and_resumes_the_session() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let _ = wait_until_stopped(&mut server);

    let resumed = server.handle_line(&request(4, "continue", json!({"task_id":99})));
    assert_eq!(resumed[0]["success"], true, "{resumed:?}");
    let terminated = server.wait();
    assert_eq!(server.status(), ServerStatus::Terminated, "{terminated:?}");
    assert!(
        terminated
            .iter()
            .any(|event| event["event"] == "output" && event["body"]["text"] == "42\n")
    );
}

#[test]
fn runtime_error_stop_identifies_the_owner_and_freezes_peers() {
    const FAILURE: &str = r#"program TaskFailure;

uses Std.Task;

procedure Explode();
begin
  panic('child boom')
end;

begin
  var Pending: task := go Explode();
  Wait(Pending)
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(FAILURE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile failure fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("debug server");
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":false})));
    let failed = wait_until_stopped(&mut server);
    assert!(
        failed
            .iter()
            .any(|record| { record["event"] == "runtime_error" && record["body"]["task_id"] == 1 })
    );
    assert!(stopped_events(&failed).iter().any(|event| {
        event["body"]["reason"] == "runtime_error"
            && event["body"]["task_id"] == 1
            && event["body"]["all_tasks_stopped"] == true
    }));
    let tasks = server.handle_line(&request(3, "tasks", json!({})));
    assert_eq!(tasks[0]["body"]["tasks"][1]["task_id"], 1);
    assert_eq!(tasks[0]["body"]["tasks"][1]["state"], "failed");
    assert_eq!(tasks[0]["body"]["tasks"][1]["inspectable"], true);
    assert_eq!(tasks[0]["body"]["tasks"][0]["inspectable"], true);
}
