//! JSONL seeded empty-storage initialization coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str =
    include_str!("../../../tests/debugger/fixtures/empty_storage_construction.fpas");

fn new_server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable =
        fpas_compiler::compile(&program).expect("compile empty-storage construction fixture");
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
    send(server, id, "scopes", json!({"frame_id": frame_id}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("locals")
}

fn named_variable(server: &mut JsonlServer, id: &mut u64, reference: u64, name: &str) -> Value {
    send(
        server,
        id,
        "variables",
        json!({"variables_reference": reference, "start": 0, "count": 20}),
    )[0]["body"]["variables"]
        .as_array()
        .expect("variables")
        .iter()
        .find(|variable| variable["name"] == name)
        .cloned()
        .unwrap_or_else(|| panic!("{name} variable"))
}

fn stop_with_empty(server: &mut JsonlServer, id: &mut u64, name: &str) -> u64 {
    for _ in 0..64 {
        let current = frame(server, id);
        let scopes = send(server, id, "scopes", json!({"frame_id": current}));
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
        let variable = named_variable(server, id, locals, name);
        match variable["value"].as_str() {
            Some("<uninitialized>") => return current,
            Some(other) => panic!("{name} already initialized as {other}"),
            None => {
                let _ = send(server, id, "step_into", json!({}));
                let _ = server.wait();
            }
        }
    }
    panic!("{name} never became visible uninitialized")
}

fn initialize(server: &mut JsonlServer) -> u64 {
    let mut id = 0;
    let initialized = send(server, &mut id, "initialize", json!({"version": 2}));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["storage_initialize"],
        true
    );
    let _ = send(server, &mut id, "launch", json!({"stop_on_entry": true}));
    id
}

#[test]
fn jsonl_storage_initialize_commits_nested_target_and_continues() {
    let mut server = new_server();
    let mut id = initialize(&mut server);
    let current = stop_with_empty(&mut server, &mut id, "State");
    let result = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({
            "frame_id": current,
            "target": "State.Nested.X",
            "initializer": "MakeInitialState()",
            "expression": "42"
        }),
    );
    assert_eq!(result[0]["success"], true, "{result:?}");
    assert_eq!(result[0]["body"]["root"], "State");
    assert_eq!(result[0]["body"]["target"], "State.Nested.X");
    assert_eq!(result[0]["body"]["value"], "42");
    assert_eq!(result[0]["body"]["type"], "integer");
    assert!(result[0]["body"]["root_value"].is_string());

    let current = frame(&mut server, &mut id);
    let locals = locals_reference(&mut server, &mut id, current);
    let state = named_variable(&mut server, &mut id, locals, "State");
    let nested = named_variable(
        &mut server,
        &mut id,
        state["variables_reference"].as_u64().expect("state"),
        "Nested",
    );
    let x = named_variable(
        &mut server,
        &mut id,
        nested["variables_reference"].as_u64().expect("nested"),
        "X",
    );
    assert_eq!(x["value"], "42");

    let current = frame(&mut server, &mut id);
    let items = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({
            "frame_id": current,
            "target": "Items[1]",
            "initializer": "[1, 2, 3]",
            "expression": "9"
        }),
    );
    assert_eq!(items[0]["success"], true, "{items:?}");
    assert_eq!(items[0]["body"]["value"], "9");

    let current = frame(&mut server, &mut id);
    let repeated = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({
            "frame_id": current,
            "target": "State.Count",
            "initializer": "MakeInitialState()",
            "expression": "1"
        }),
    );
    assert_eq!(
        repeated[0]["error"]["code"], "storage_already_initialized",
        "{repeated:?}"
    );

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .map(|record| record["body"]["text"].as_str().unwrap_or_default())
        .collect::<String>();
    assert_eq!(
        output, "1\n42\n1\n",
        "exact source initializer suppression preserves the seeded root: {output:?}"
    );
}

#[test]
fn jsonl_storage_initialize_rejects_strict_arguments_and_preserves_empty_storage() {
    let mut server = new_server();
    let mut id = initialize(&mut server);
    let current = stop_with_empty(&mut server, &mut id, "State");
    let extra = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({
            "frame_id": current,
            "target": "State.Count",
            "initializer": "MakeInitialState()",
            "expression": "1",
            "extra": true
        }),
    );
    assert_eq!(extra[0]["error"]["code"], "invalid_request");

    let missing = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({"frame_id": current, "target": "State.Count", "initializer": "MakeInitialState()"}),
    );
    assert_eq!(missing[0]["error"]["code"], "invalid_request");

    let wrong_type = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({
            "frame_id": current,
            "target": "State.Count",
            "initializer": "MakeInitialState()",
            "expression": 42
        }),
    );
    assert_eq!(wrong_type[0]["error"]["code"], "invalid_request");

    let parse = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({
            "frame_id": current,
            "target": "State.Count",
            "initializer": "MakeInitialState(",
            "expression": "1"
        }),
    );
    assert_eq!(parse[0]["success"], false);
    assert!(parse[0]["error"]["offset"].is_number(), "{parse:?}");

    let root_only = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({
            "frame_id": current,
            "target": "State",
            "initializer": "MakeInitialState()",
            "expression": "MakeInitialState()"
        }),
    );
    assert_eq!(root_only[0]["error"]["code"], "variable_path_unsupported");

    let locals = locals_reference(&mut server, &mut id, current);
    assert_eq!(
        named_variable(&mut server, &mut id, locals, "State")["value"],
        "<uninitialized>"
    );

    let mut running = new_server();
    let _ = running.handle_line(&request(1, "initialize", json!({"version": 2})));
    let _ = running.handle_line(&request(2, "launch", json!({"stop_on_entry": false})));
    let invalid = running.handle_line(&request(
        3,
        "storage.initialize",
        json!({
            "target": "State.Count",
            "initializer": "MakeInitialState()",
            "expression": "1"
        }),
    ));
    assert_eq!(invalid[0]["error"]["code"], "invalid_state");
    let _ = running.handle_line(&request(4, "disconnect", json!({})));
}

#[test]
fn jsonl_storage_initialize_stays_bound_to_the_selected_task() {
    let mut server = new_server();
    let mut id = initialize(&mut server);
    let current = stop_with_empty(&mut server, &mut id, "State");
    let foreign = send(
        &mut server,
        &mut id,
        "storage.initialize",
        json!({
            "frame_id": current.saturating_add(1),
            "target": "State.Count",
            "initializer": "MakeInitialState()",
            "expression": "1"
        }),
    );
    assert_eq!(foreign[0]["error"]["code"], "unknown_frame", "{foreign:?}");
}
