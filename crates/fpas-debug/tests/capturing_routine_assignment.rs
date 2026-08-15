//! JSONL capturing named-routine assignment coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str =
    include_str!("../../../tests/debugger/fixtures/capturing_routine_assignment.fpas");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile capturing-routine fixture");
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

fn stop_at(server: &mut JsonlServer, id: &mut u64, needle: &str) -> u64 {
    let line = SOURCE
        .lines()
        .position(|line| line.contains(needle))
        .expect("marker")
        + 1;
    let breakpoint = send(
        server,
        id,
        "breakpoint.set",
        json!({"source":"<memory>","line":line}),
    );
    assert_eq!(breakpoint[0]["body"]["verified"], true, "{breakpoint:?}");
    let _ = send(server, id, "continue", json!({}));
    let _ = server.wait();
    frame(server, id)
}

#[test]
fn jsonl_capturing_nested_routines_assign_from_the_owner_frame() {
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
    let owner = stop_at(&mut server, &mut id, "var MakeStop: integer := 0;");
    let locals = locals_reference(&mut server, &mut id, owner);

    let assigned = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":locals,"name":"Current","expression":"AddBase"}),
    );
    assert_eq!(assigned[0]["success"], true, "{assigned:?}");
    assert_eq!(
        assigned[0]["body"]["result"],
        "<function makeadder.addbase>"
    );

    let current = frame(&mut server, &mut id);
    let invoked = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id":current,"expression":"Current(1)"}),
    );
    assert_eq!(invoked[0]["success"], true, "{invoked:?}");
    assert_eq!(invoked[0]["body"]["result"], "11");
}

#[test]
fn jsonl_qualified_nested_name_and_cell_rejection_share_the_engine_contract() {
    let mut jsonl = server();
    let mut id = 0;
    let _ = send(&mut jsonl, &mut id, "initialize", json!({"version":2}));
    let _ = send(&mut jsonl, &mut id, "launch", json!({"stop_on_entry":true}));
    let owner = stop_at(&mut jsonl, &mut id, "var MakeStop: integer := 0;");
    let qualified = send(
        &mut jsonl,
        &mut id,
        "expression.set",
        json!({"frame_id":owner,"target":"Current","expression":"MakeAdder.AddBase"}),
    );
    assert_eq!(qualified[0]["success"], true, "{qualified:?}");
    assert_eq!(
        qualified[0]["body"]["result"],
        "<function makeadder.addbase>"
    );

    let mut cell_server = server();
    let mut cell_id = 0;
    let _ = send(
        &mut cell_server,
        &mut cell_id,
        "initialize",
        json!({"version":2}),
    );
    let _ = send(
        &mut cell_server,
        &mut cell_id,
        "launch",
        json!({"stop_on_entry":true}),
    );
    let cell_frame = stop_at(
        &mut cell_server,
        &mut cell_id,
        "var CellStop: integer := 0;",
    );
    let locals = locals_reference(&mut cell_server, &mut cell_id, cell_frame);
    let rejected = send(
        &mut cell_server,
        &mut cell_id,
        "variable.set",
        json!({"variables_reference":locals,"name":"Current","expression":"AddCell"}),
    );
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected[0]["error"]["code"], "variable_value_type");
    let current = frame(&mut cell_server, &mut cell_id);
    let preserved = send(
        &mut cell_server,
        &mut cell_id,
        "evaluate",
        json!({"frame_id":current,"expression":"Current(1)"}),
    );
    assert_eq!(preserved[0]["body"]["result"], "1");
}
