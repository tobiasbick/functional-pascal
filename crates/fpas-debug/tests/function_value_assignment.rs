//! JSONL function-value assignment coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str =
    include_str!("../../../tests/debugger/fixtures/function_value_assignment.fpas");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile function-value fixture");
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
    panic!("function-value fixture locals never became initialized")
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

#[test]
fn jsonl_function_values_copy_atomically_and_continue() {
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

    let copied = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":initial_frame,"target":"Current","expression":"Backup"}),
    );
    assert_eq!(copied[0]["success"], true, "{copied:?}");
    assert_eq!(copied[0]["body"]["result"], "<function addtwo>");

    let current = frame(&mut server, &mut id);
    let invoked = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id":current,"expression":"Current(1)"}),
    );
    assert_eq!(invoked[0]["success"], true, "{invoked:?}");
    assert_eq!(invoked[0]["body"]["result"], "3");

    let stale_locals = locals_reference(&mut server, &mut id, current);
    let captured = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":stale_locals,"name":"Current","expression":"Captured"}),
    );
    assert_eq!(captured[0]["success"], true, "{captured:?}");
    assert!(
        captured[0]["body"]["result"]
            .as_str()
            .expect("result")
            .starts_with("<function"),
        "{captured:?}"
    );

    let current = frame(&mut server, &mut id);
    let captured_call = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id":current,"expression":"Current(1)"}),
    );
    assert_eq!(captured_call[0]["body"]["result"], "11");

    let assigned = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Current","expression":"AddTwo"}),
    );
    assert_eq!(assigned[0]["success"], true, "{assigned:?}");
    assert_eq!(assigned[0]["body"]["result"], "<function addtwo>");
    let current = frame(&mut server, &mut id);
    let named_call = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id":current,"expression":"Current(1)"}),
    );
    assert_eq!(named_call[0]["body"]["result"], "3");

    let qualified = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Current","expression":"Math.Transform"}),
    );
    assert_eq!(qualified[0]["success"], true, "{qualified:?}");
    assert_eq!(qualified[0]["body"]["result"], "<function math.transform>");
    let current = frame(&mut server, &mut id);
    let qualified_call = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id":current,"expression":"Current(1)"}),
    );
    assert_eq!(qualified_call[0]["body"]["result"], "4");

    let current = frame(&mut server, &mut id);
    let bound = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Current","expression":"Receiver.Add"}),
    );
    assert_eq!(bound[0]["success"], true, "{bound:?}");
    assert_eq!(bound[0]["body"]["result"], "<function Counter.Add>");
    let current = frame(&mut server, &mut id);
    let bound_call = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id":current,"expression":"Current(1)"}),
    );
    assert_eq!(bound_call[0]["body"]["result"], "11", "{bound_call:?}");
    let reset = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Current","expression":"Math.Transform"}),
    );
    assert_eq!(reset[0]["success"], true, "{reset:?}");
    let current = frame(&mut server, &mut id);

    let failures = [
        ("Frozen", "Backup", "variable_not_mutable"),
        ("Current", "Transform", "call_ambiguous"),
        ("Current", "MakeAdder(1)", "variable_value_type"),
        (
            "Current",
            "function(Value: integer): integer begin return Value end",
            "unsupported_expression",
        ),
        ("Current", "1", "variable_value_type"),
        ("Current", "Receiver.Missing", "unknown_name"),
        ("WrongSignature", "Receiver.Add", "variable_value_type"),
        ("Current", "MissingRoutine", "unknown_name"),
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
    }
    let stale = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":stale_locals,"name":"Current","expression":"Backup"}),
    );
    assert_eq!(stale[0]["error"]["code"], "variable_target_expired");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "4\n11\n");
}

#[test]
fn jsonl_function_value_assignment_stays_bound_to_the_selected_child_task() {
    const TASK_SOURCE: &str = r#"program TaskFunctionValueAssignment;

uses Std.Console, Std.Task;

type
  Handler = function(Value: integer): integer;

function AddOne(Value: integer): integer;
begin
  return Value + 1
end;

function AddTwo(Value: integer): integer;
begin
  return Value + 2
end;

function Work(): integer;
begin
  mutable var Current: Handler := AddOne;
  var Backup: Handler := AddTwo;
  var Marker: integer := 0;
  return Current(1)
end;

begin
  var Pending: task := go Work();
  WriteLn(Wait(Pending))
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(TASK_SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task function-value fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server");
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

    let updated = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":child_frame,"target":"Current","expression":"AddTwo"}),
    );
    assert_eq!(
        updated[0]["body"]["result"], "<function addtwo>",
        "{updated:?}"
    );
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
            .any(|record| record["event"] == "output" && record["body"]["text"] == "3\n"),
        "{terminated:?}"
    );
}
