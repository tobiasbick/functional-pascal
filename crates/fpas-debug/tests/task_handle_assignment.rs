//! JSONL task-handle assignment coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/task_handle_assignment.fpas");

fn server() -> JsonlServer {
    server_for(SOURCE)
}

fn server_for(source: &str) -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task-handle fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn send(server: &mut JsonlServer, id: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *id += 1;
    server.handle_line(&request(*id, command, arguments))
}

fn frame(server: &mut JsonlServer, id: &mut u64) -> u64 {
    send(server, id, "stack", json!({}))[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("frame ID")
}

fn stop_with_initialized_locals(server: &mut JsonlServer, id: &mut u64) -> u64 {
    for _ in 0..64 {
        let current = frame(server, id);
        let ready = send(
            server,
            id,
            "evaluate",
            json!({"frame_id":current,"expression":"StopMarker"}),
        );
        if ready[0]["success"] == true {
            return current;
        }
        let _ = send(server, id, "step_into", json!({}));
        let _ = server.wait();
    }
    panic!("task-handle fixture locals never became initialized")
}

fn locals_reference(server: &mut JsonlServer, id: &mut u64, frame_id: u64) -> u64 {
    send(server, id, "scopes", json!({"frame_id":frame_id}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("locals")
}

fn evaluate_result(
    server: &mut JsonlServer,
    id: &mut u64,
    frame_id: u64,
    expression: &str,
) -> String {
    send(
        server,
        id,
        "evaluate",
        json!({"frame_id":frame_id,"expression":expression}),
    )[0]["body"]["result"]
        .as_str()
        .expect("result")
        .to_string()
}

#[test]
fn jsonl_task_handles_copy_atomically_and_continue_through_wait() {
    let mut server = server();
    let mut id = 0;
    let initialized = send(&mut server, &mut id, "initialize", json!({"version":2}));
    assert_eq!(initialized[0]["body"]["capabilities"]["set_variable"], true);
    assert_eq!(
        initialized[0]["body"]["capabilities"]["set_expression"],
        true
    );
    let _ = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":true}),
    );
    let initial_frame = stop_with_initialized_locals(&mut server, &mut id);
    let pending = evaluate_result(&mut server, &mut id, initial_frame, "Pending");
    assert!(pending.starts_with("<task "), "{pending}");

    let copied = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":initial_frame,"target":"Current","expression":"Pending"}),
    );
    assert_eq!(copied[0]["success"], true, "{copied:?}");
    assert_eq!(copied[0]["body"]["result"], pending);

    let current = frame(&mut server, &mut id);
    let locals = locals_reference(&mut server, &mut id, current);
    let handle = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":locals,"name":"Current","expression":"Pending"}),
    );
    assert_eq!(handle[0]["success"], true, "{handle:?}");
    assert_eq!(handle[0]["body"]["result"], pending);

    let current = frame(&mut server, &mut id);
    let failures = [
        ("Frozen", "Pending", "variable_not_mutable"),
        ("Current", "Wrong", "variable_value_type"),
        ("Current", "1", "variable_value_type"),
        ("Current", "'<task 1>'", "variable_value_type"),
        ("Current", "Seven()", "variable_value_type"),
        ("Current", "MissingName", "unknown_name"),
    ];
    for (target, expression, code) in failures {
        let failed = send(
            &mut server,
            &mut id,
            "expression.set",
            json!({"frame_id":current,"target":target,"expression":expression}),
        );
        assert_eq!(
            failed[0]["success"], false,
            "{target} := {expression}: {failed:?}"
        );
        assert_eq!(
            failed[0]["error"]["code"], code,
            "{target} := {expression}: {failed:?}"
        );
        let help = failed[0]["error"]["help"].as_str().unwrap_or_default();
        assert!(
            !help.contains("enter a numeric ID") || help.contains("Do not enter"),
            "{target} := {expression}: {failed:?}"
        );
    }
    let stale = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":locals,"name":"Current","expression":"Pending"}),
    );
    assert_eq!(stale[0]["error"]["code"], "variable_target_expired");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "7\n");
}

#[test]
fn jsonl_task_handle_assignment_stays_bound_to_the_selected_child_task() {
    const TASK_SOURCE: &str = r#"program TaskHandleChildAssignment;

uses Std.Console, Std.Task;

function Seven(): integer;
begin
  return 7
end;

function Nine(): integer;
begin
  return 9
end;

function Work(): integer;
begin
  var Backup: task := go Seven();
  mutable var Current: task := go Nine();
  var Marker: integer := 0;
  return Wait(Current)
end;

begin
  var Pending: task := go Work();
  WriteLn(Wait(Pending))
end.
"#;
    let mut server = server_for(TASK_SOURCE);
    let mut id = 0;
    let _ = send(&mut server, &mut id, "initialize", json!({"version":2}));
    let marker_line = TASK_SOURCE
        .lines()
        .position(|line| line.contains("var Marker: integer := 0;"))
        .expect("marker line")
        + 1;
    let breakpoint = send(
        &mut server,
        &mut id,
        "breakpoint.set",
        json!({"source":"<memory>","line":marker_line}),
    );
    assert_eq!(breakpoint[0]["body"]["verified"], true, "{breakpoint:?}");
    let _ = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":false}),
    );
    let stopped = server.wait();
    assert!(
        stopped
            .iter()
            .any(|record| record["event"] == "stopped" && record["body"]["task_id"] == 1),
        "{stopped:?}"
    );

    let child_stack = send(&mut server, &mut id, "stack", json!({"task_id":1}));
    let child_frame = child_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("child frame");
    let main_stack = send(&mut server, &mut id, "stack", json!({"task_id":0}));
    let main_frame = main_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("main frame");
    let backup = evaluate_result(&mut server, &mut id, child_frame, "Backup");

    let updated = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":child_frame,"target":"Current","expression":"Backup"}),
    );
    assert_eq!(updated[0]["body"]["result"], backup, "{updated:?}");
    let expired_main = send(
        &mut server,
        &mut id,
        "scopes",
        json!({"frame_id":main_frame}),
    );
    assert_eq!(expired_main[0]["error"]["code"], "unknown_frame");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let terminated = server.wait();
    assert!(
        terminated
            .iter()
            .any(|record| record["event"] == "output" && record["body"]["text"] == "7\n"),
        "{terminated:?}"
    );
}
