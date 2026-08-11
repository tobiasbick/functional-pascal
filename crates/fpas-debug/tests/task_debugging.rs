//! JSONL task enumeration, selection, stepping, and lifecycle coverage.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus},
};
use serde_json::{Value, json};

const SOURCE: &str = r#"program TaskDebugging;

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
    server_for(SOURCE)
}

fn server_for(source: &str) -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("debug server")
}

fn run_to_completion(server: &mut JsonlServer) -> Vec<Value> {
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":false})));
    let records = server.wait();
    assert_eq!(server.status(), ServerStatus::Terminated, "{records:?}");
    records
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn wait_until_stopped(server: &mut JsonlServer) -> Vec<Value> {
    let records = server.wait();
    assert_eq!(server.status(), ServerStatus::Stopped, "{records:?}");
    records
}

#[test]
fn child_breakpoint_exposes_task_catalog_stack_and_selected_step() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(initialized[0]["body"]["capabilities"]["task_threads"], true);
    let breakpoint = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    assert_eq!(breakpoint[0]["body"]["verified"], true, "{breakpoint:?}");
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));

    let events = wait_until_stopped(&mut server);
    assert!(events.iter().any(|event| {
        event["event"] == "task"
            && event["body"]["reason"] == "started"
            && event["body"]["task_id"] == 1
    }));
    assert!(events.iter().any(|event| {
        event["event"] == "stopped"
            && event["body"]["task_id"] == 1
            && event["body"]["all_tasks_stopped"] == true
    }));

    let tasks = server.handle_line(&request(4, "tasks", json!({})));
    assert_eq!(tasks[0]["body"]["total"], 2);
    assert_eq!(tasks[0]["body"]["tasks"][0]["task_id"], 0);
    assert_eq!(tasks[0]["body"]["tasks"][1]["task_id"], 1);

    let stack = server.handle_line(&request(5, "stack", json!({"task_id":1})));
    assert_eq!(stack[0]["body"]["task_id"], 1);
    assert_eq!(stack[0]["body"]["frames"][0]["name"], "work");

    let step = server.handle_line(&request(6, "step_into", json!({"task_id":1})));
    assert_eq!(step[0]["success"], true, "{step:?}");
    let step_events = wait_until_stopped(&mut server);
    assert!(step_events.iter().any(|event| {
        event["event"] == "stopped"
            && event["body"]["reason"] == "step"
            && event["body"]["task_id"] == 1
    }));

    let _ = server.handle_line(&request(7, "continue", json!({})));
    let terminated = server.wait();
    assert_eq!(server.status(), ServerStatus::Terminated, "{terminated:?}");
    assert!(
        terminated
            .iter()
            .any(|event| { event["event"] == "output" && event["body"]["text"] == "42\n" })
    );
}

#[test]
fn unknown_task_selection_returns_a_stable_error() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));

    let stack = server.handle_line(&request(3, "stack", json!({"task_id":99})));

    assert_eq!(stack[0]["error"]["code"], "unknown_task");
    assert!(
        stack[0]["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("99"))
    );

    let invalid_stack = server.handle_line(&request(4, "stack", json!({"task_id":"main"})));
    assert_eq!(invalid_stack[0]["error"]["code"], "invalid_request");
    assert!(
        invalid_stack[0]["error"]["help"]
            .as_str()
            .is_some_and(|help| help.contains("returned by `tasks`"))
    );

    let invalid_step = server.handle_line(&request(5, "step_into", json!({"task_id":-1})));
    assert_eq!(invalid_step[0]["error"]["code"], "invalid_request");

    let unknown_step = server.handle_line(&request(6, "step_into", json!({"task_id":99})));
    assert_eq!(unknown_step[0]["error"]["code"], "unknown_task");
    assert_eq!(server.status(), ServerStatus::Stopped);
}

#[test]
fn task_snapshots_and_mutation_remain_bound_to_their_task() {
    let mut server = server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let _ = wait_until_stopped(&mut server);

    let child_stack = server.handle_line(&request(4, "stack", json!({"task_id":1})));
    let child_frame = child_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("child frame");
    let main_stack = server.handle_line(&request(5, "stack", json!({"task_id":0})));
    let main_frame = main_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("main frame");
    assert_ne!(child_frame >> 32, main_frame >> 32);

    let scopes = server.handle_line(&request(6, "scopes", json!({"frame_id":child_frame})));
    let locals = scopes[0]["body"]["scopes"]
        .as_array()
        .expect("child scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("child locals");
    let variables = server.handle_line(&request(
        7,
        "variables",
        json!({"variables_reference":locals}),
    ));
    assert!(
        variables[0]["body"]["variables"]
            .as_array()
            .expect("child variables")
            .iter()
            .any(|variable| variable["name"] == "Value" && variable["value"] == "40")
    );

    let mutation = server.handle_line(&request(
        8,
        "variable.set",
        json!({"variables_reference":locals,"name":"Value","expression":"41"}),
    ));
    assert_eq!(mutation[0]["body"]["result"], "41", "{mutation:?}");

    let refreshed = server.handle_line(&request(9, "stack", json!({"task_id":1})));
    let refreshed_frame = refreshed[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("refreshed child frame");
    let evaluated = server.handle_line(&request(
        10,
        "evaluate",
        json!({"frame_id":refreshed_frame,"expression":"Value"}),
    ));
    assert_eq!(evaluated[0]["body"]["result"], "41", "{evaluated:?}");
    let expired = server.handle_line(&request(11, "scopes", json!({"frame_id":child_frame})));
    assert_eq!(expired[0]["error"]["code"], "unknown_frame");

    let _ = server.handle_line(&request(12, "continue", json!({})));
    let terminated = server.wait();
    assert!(
        terminated
            .iter()
            .any(|event| { event["event"] == "output" && event["body"]["text"] == "43\n" })
    );
}

#[test]
fn wait_all_sleep_detached_and_nested_spawn_complete_under_debugging() {
    const WAIT_ALL: &str = r#"program TaskWaitAll;

uses Std.Console, Std.Task, Std.Time;

function Work(Value: integer): integer;
begin
  Sleep(1);
  return Value
end;

procedure Detached();
begin
end;

begin
  go Detached();
  var First: task := go Work(20);
  var Second: task := go Work(22);
  var Pending: array of task := [First, Second, First];
  WaitAll(Pending);
  WriteLn(Wait(First));
  WriteLn(Wait(Second))
end.
"#;
    const NESTED: &str = r#"program NestedTasks;

uses Std.Console, Std.Task;

function Leaf(Value: integer): integer;
begin
  return Value
end;

function Parent(): integer;
begin
  var Child: task := go Leaf(41);
  return Wait(Child) + 1
end;

begin
  var Pending: task := go Parent();
  WriteLn(Wait(Pending))
end.
"#;

    let mut wait_all = server_for(WAIT_ALL);
    let records = run_to_completion(&mut wait_all);
    let output = records
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "20\n22\n");
    let started = records
        .iter()
        .filter(|record| record["event"] == "task" && record["body"]["reason"] == "started")
        .count();
    let exited = records
        .iter()
        .filter(|record| record["event"] == "task" && record["body"]["reason"] == "exited")
        .count();
    assert_eq!((started, exited), (3, 3));

    let mut nested = server_for(NESTED);
    let records = run_to_completion(&mut nested);
    assert!(
        records
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "42\n" })
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| { record["event"] == "task" && record["body"]["reason"] == "started" })
            .count(),
        2
    );
}

#[test]
fn child_failure_and_root_shutdown_report_one_clean_lifecycle() {
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
    const SHUTDOWN: &str = r#"program DetachedShutdown;

uses Std.Console, Std.Task, Std.Time;

procedure Later();
begin
  Sleep(1000);
  WriteLn('too late')
end;

begin
  go Later()
end.
"#;

    let mut failure = server_for(FAILURE);
    let _ = failure.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = failure.handle_line(&request(2, "launch", json!({"stop_on_entry":false})));
    let failed = wait_until_stopped(&mut failure);
    let runtime_errors = failed
        .iter()
        .filter(|record| record["event"] == "runtime_error")
        .collect::<Vec<_>>();
    assert_eq!(runtime_errors.len(), 1, "{failed:?}");
    assert_eq!(runtime_errors[0]["body"]["task_id"], 1);
    assert!(
        runtime_errors[0]["body"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("child boom"))
    );
    let disconnected = failure.handle_line(&request(3, "disconnect", json!({})));
    assert_eq!(
        disconnected
            .iter()
            .filter(|record| {
                record["event"] == "task"
                    && record["body"]["reason"] == "exited"
                    && record["body"]["task_id"] == 1
            })
            .count(),
        1
    );

    let mut shutdown = server_for(SHUTDOWN);
    let records = run_to_completion(&mut shutdown);
    assert!(!records.iter().any(|record| record["event"] == "output"));
    assert_eq!(
        records
            .iter()
            .filter(|record| { record["event"] == "task" && record["body"]["reason"] == "exited" })
            .count(),
        1
    );
}
