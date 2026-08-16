//! JSONL task cancel plus rejected create/restart and history.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TaskLifecycle;

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
    let executable = fpas_compiler::compile(&program).expect("compile task-lifecycle fixture");
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

fn child<'a>(tasks: &'a Value) -> &'a Value {
    tasks["body"]["tasks"]
        .as_array()
        .expect("tasks")
        .iter()
        .find(|task| task["task_id"] == 1)
        .expect("child")
}

#[test]
fn initialize_advertises_cancel_without_create_restart_non_stop_or_history() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let capabilities = &initialized[0]["body"]["capabilities"];
    assert_eq!(capabilities["task_cancel"], true);
    assert_eq!(capabilities["task_create"], false);
    assert_eq!(capabilities["task_restart"], false);
    assert_eq!(capabilities["task_pause"], true);
    assert_eq!(capabilities["non_stop"], false);
    assert_eq!(capabilities["reverse_execution"], false);
    assert!(capabilities.get("task_history").is_none());

    let history = server.handle_line(&request(2, "task.history", json!({})));
    assert_eq!(history[0]["success"], false, "{history:?}");
    assert_eq!(history[0]["error"]["code"], "unsupported_capability");
}

#[test]
fn cancel_requires_a_current_task_id_and_rejects_root() {
    let mut server = server();
    stop_in_child(&mut server);
    let before = server.handle_line(&request(4, "tasks", json!({})));
    assert_eq!(child(&before[0])["state"], "runnable");

    let missing = server.handle_line(&request(5, "task.cancel", json!({})));
    assert_eq!(missing[0]["success"], false, "{missing:?}");
    assert_eq!(missing[0]["error"]["code"], "invalid_request");

    let unknown = server.handle_line(&request(6, "task.cancel", json!({"task_id":99})));
    assert_eq!(unknown[0]["success"], false, "{unknown:?}");
    assert_eq!(unknown[0]["error"]["code"], "unknown_task");

    let root = server.handle_line(&request(7, "task.cancel", json!({"task_id":0})));
    assert_eq!(root[0]["success"], false, "{root:?}");
    assert_eq!(root[0]["error"]["code"], "invalid_state");

    let after = server.handle_line(&request(8, "tasks", json!({})));
    assert_eq!(after[0]["body"], before[0]["body"]);
}

#[test]
fn cancel_child_emits_exit_and_continue_observes_waiter_failure() {
    let mut server = server();
    stop_in_child(&mut server);

    let cancelled = server.handle_line(&request(4, "task.cancel", json!({"task_id":1})));
    assert_eq!(cancelled[0]["success"], true, "{cancelled:?}");
    assert_eq!(cancelled[0]["body"]["task_id"], 1);
    assert_eq!(cancelled[0]["body"]["state"], "cancelled");
    assert!(cancelled.iter().any(|record| {
        record["event"] == "task"
            && record["body"]["reason"] == "exited"
            && record["body"]["task_id"] == 1
    }));
    assert_eq!(server.status(), ServerStatus::Stopped);

    let catalog = server.handle_line(&request(5, "tasks", json!({})));
    assert_eq!(child(&catalog[0])["state"], "cancelled");
    assert_eq!(child(&catalog[0])["inspectable"], false);

    let _ = server.handle_line(&request(6, "continue", json!({})));
    let stopped = wait_until_stopped(&mut server);
    assert!(stopped.iter().any(|record| {
        record["event"] == "stopped" && record["body"]["reason"] == "runtime_error"
    }));
    assert!(
        stopped.iter().any(|record| {
            record["event"] == "runtime_error" && record["body"]["code"] == "F4016"
        })
    );
}

#[test]
fn create_and_restart_are_advertised_false_and_reject_atomically() {
    let mut server = server();
    stop_in_child(&mut server);
    let before = server.handle_line(&request(4, "tasks", json!({})));

    let created = server.handle_line(&request(5, "task.create", json!({})));
    assert_eq!(created[0]["success"], false, "{created:?}");
    assert_eq!(created[0]["error"]["code"], "task_create_unsupported");

    let restarted = server.handle_line(&request(6, "task.restart", json!({"task_id":1})));
    assert_eq!(restarted[0]["success"], false, "{restarted:?}");
    assert_eq!(restarted[0]["error"]["code"], "task_restart_unsupported");

    let unknown = server.handle_line(&request(7, "task.restart", json!({"task_id":99})));
    assert_eq!(unknown[0]["success"], false, "{unknown:?}");
    assert_eq!(unknown[0]["error"]["code"], "unknown_task");

    let after = server.handle_line(&request(8, "tasks", json!({})));
    assert_eq!(after[0]["body"], before[0]["body"]);
}
