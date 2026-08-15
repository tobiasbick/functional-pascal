//! JSONL Cell and EnclosingCell named-routine assignment coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str =
    include_str!("../../../tests/debugger/fixtures/cell_capturing_routine_assignment.fpas");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile cell-capturing fixture");
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
fn jsonl_cell_capturing_assignment_continues_through_shared_cells() {
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
    let owner = stop_at(&mut server, &mut id, "var CellStop: integer := 0;");
    let locals = locals_reference(&mut server, &mut id, owner);

    let assigned = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":locals,"name":"Current","expression":"AddCell"}),
    );
    assert_eq!(assigned[0]["success"], true, "{assigned:?}");
    assert_eq!(assigned[0]["body"]["result"], "<function mutating.addcell>");

    let current = frame(&mut server, &mut id);
    let qualified = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Current","expression":"Mutating.AddCell"}),
    );
    assert_eq!(qualified[0]["success"], true, "{qualified:?}");
    assert_eq!(
        qualified[0]["body"]["result"],
        "<function mutating.addcell>"
    );

    let current = frame(&mut server, &mut id);
    let copied = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Copy","expression":"Current"}),
    );
    assert_eq!(copied[0]["success"], true, "{copied:?}");
    assert_eq!(copied[0]["body"]["result"], "<function mutating.addcell>");

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert!(
        output.starts_with("12\n13\n"),
        "continuation must observe shared cell writes, got {output:?}"
    );
}

#[test]
fn jsonl_global_cell_destination_is_rejected_without_mutation() {
    let mut server = server();
    let mut id = 0;
    let _ = send(&mut server, &mut id, "initialize", json!({"version":2}));
    let _ = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":true}),
    );
    let owner = stop_at(&mut server, &mut id, "var CellStop: integer := 0;");
    let rejected = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":owner,"target":"Shared","expression":"AddCell"}),
    );
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert_eq!(rejected[0]["error"]["code"], "variable_value_type");
    let current = frame(&mut server, &mut id);
    let preserved = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id":current,"expression":"Current(1)"}),
    );
    assert_eq!(preserved[0]["body"]["result"], "1");
}
