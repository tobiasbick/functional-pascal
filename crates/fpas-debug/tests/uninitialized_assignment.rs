//! JSONL complete initialization of uninitialized mutable roots.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/uninitialized_assignment.fpas");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile uninitialized fixture");
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

fn locals_reference(server: &mut JsonlServer, id: &mut u64, frame_id: u64) -> u64 {
    send(server, id, "scopes", json!({"frame_id":frame_id}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("locals")
}

fn local_value(server: &mut JsonlServer, id: &mut u64, name: &str) -> String {
    let current = frame(server, id);
    let locals = locals_reference(server, id, current);
    send(
        server,
        id,
        "variables",
        json!({"variables_reference":locals,"start":0,"count":20}),
    )[0]["body"]["variables"]
        .as_array()
        .expect("locals")
        .iter()
        .find(|value| value["name"] == name)
        .and_then(|value| value["value"].as_str())
        .expect(name)
        .to_string()
}

fn stop_with_uninitialized_count(server: &mut JsonlServer, id: &mut u64) -> u64 {
    for _ in 0..64 {
        let current = frame(server, id);
        let scopes = send(server, id, "scopes", json!({"frame_id":current}));
        let Some(locals) = scopes[0]["body"]["scopes"]
            .as_array()
            .expect("scopes")
            .iter()
            .find(|scope| scope["name"] == "Locals")
            .and_then(|scope| scope["variables_reference"].as_u64())
        else {
            let _ = send(server, id, "step_into", json!({}));
            let _ = server.wait();
            continue;
        };
        let variables = send(
            server,
            id,
            "variables",
            json!({"variables_reference":locals,"start":0,"count":20}),
        );
        let count = variables[0]["body"]["variables"]
            .as_array()
            .expect("locals")
            .iter()
            .find(|value| value["name"] == "Count");
        match count.and_then(|value| value["value"].as_str()) {
            Some("<uninitialized>") => return current,
            Some(other) => panic!("Count already initialized as {other}"),
            None => {
                let _ = send(server, id, "step_into", json!({}));
                let _ = server.wait();
            }
        }
    }
    panic!("uninitialized Count never became visible")
}

#[test]
fn jsonl_uninitialized_roots_initialize_atomically_and_continue() {
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
    let initial_frame = stop_with_uninitialized_count(&mut server, &mut id);
    assert_eq!(
        local_value(&mut server, &mut id, "Count"),
        "<uninitialized>"
    );

    let locals = locals_reference(&mut server, &mut id, initial_frame);
    let count = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":locals,"name":"Count","expression":"30"}),
    );
    assert_eq!(count[0]["body"]["result"], "30", "{count:?}");
    assert_eq!(local_value(&mut server, &mut id, "Count"), "30");

    let current = frame(&mut server, &mut id);
    let flag = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"fLaG","expression":"true"}),
    );
    assert_eq!(flag[0]["body"]["result"], "true", "{flag:?}");

    let global = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"target":"GlobalCount","expression":"8"}),
    );
    assert_eq!(global[0]["body"]["result"], "8", "{global:?}");

    let current = frame(&mut server, &mut id);
    for (target, expression, code) in [
        ("Count", "true", "variable_value_type"),
        ("Frozen", "9", "variable_not_mutable"),
        ("Origin.X", "1", "variable_path_unsupported"),
    ] {
        let failed = send(
            &mut server,
            &mut id,
            "expression.set",
            json!({"frame_id":current,"target":target,"expression":expression}),
        );
        assert_eq!(
            failed[0]["error"]["code"], code,
            "{target} {expression}: {failed:?}"
        );
        assert_eq!(local_value(&mut server, &mut id, "Count"), "30");
    }

    let stale = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":locals,"name":"Count","expression":"1"}),
    );
    assert_eq!(stale[0]["error"]["code"], "variable_target_expired");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "1\n99\n0\n3\n2\n");
}

#[test]
fn jsonl_uninitialized_assignment_stays_bound_to_the_selected_child_task() {
    const TASK_SOURCE: &str = r#"program TaskUninitializedAssignment;

uses Std.Console, Std.Task;

function Work(): integer;
begin
  mutable var Count: integer := 1;
  var Marker: integer := 0;
  return Count
end;

begin
  var Pending: task := go Work();
  WriteLn(Wait(Pending))
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(TASK_SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server");
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
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
            .any(|record| { record["event"] == "stopped" && record["body"]["task_id"] == 1 })
    );

    let child_stack = server.handle_line(&request(4, "stack", json!({"task_id":1})));
    let child_frame = child_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("child frame");
    let child_scopes = server.handle_line(&request(5, "scopes", json!({"frame_id":child_frame})));
    let child_locals = child_scopes[0]["body"]["scopes"]
        .as_array()
        .expect("child scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("child locals");
    let child_variables = server.handle_line(&request(
        6,
        "variables",
        json!({"variables_reference":child_locals,"start":0,"count":20}),
    ));
    assert_eq!(
        child_variables[0]["body"]["variables"][0]["value"], "<uninitialized>",
        "{child_variables:?}"
    );
    let main_stack = server.handle_line(&request(7, "stack", json!({"task_id":0})));
    let main_frame = main_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("main frame");

    let updated = server.handle_line(&request(
        8,
        "expression.set",
        json!({"frame_id":child_frame,"target":"Count","expression":"5"}),
    ));
    assert_eq!(updated[0]["body"]["result"], "5", "{updated:?}");
    let expired_main = server.handle_line(&request(9, "scopes", json!({"frame_id":main_frame})));
    assert_eq!(expired_main[0]["error"]["code"], "unknown_frame");

    let _ = server.handle_line(&request(10, "continue", json!({})));
    let terminated = server.wait();
    assert!(
        terminated
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "1\n" })
    );
}
