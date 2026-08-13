//! JSONL variant discovery and construction coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/variant_construction.fpas");

#[path = "variant_construction/construction.rs"]
mod construction;
#[path = "variant_construction/discovery.rs"]
mod discovery;
#[path = "variant_construction/rejection.rs"]
mod rejection;

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable =
        fpas_compiler::compile(&program).expect("compile variant construction fixture");
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
    panic!("variant construction fixture locals never became initialized")
}

fn initialize(server: &mut JsonlServer) -> u64 {
    let mut id = 0;
    let initialized = send(server, &mut id, "initialize", json!({"version": 2}));
    assert_eq!(
        initialized[0]["body"]["capabilities"]["variant_describe"],
        true
    );
    assert_eq!(
        initialized[0]["body"]["capabilities"]["variant_construct"],
        true
    );
    let _ = send(server, &mut id, "launch", json!({"stop_on_entry": true}));
    id
}
