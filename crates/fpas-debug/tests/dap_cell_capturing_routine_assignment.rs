//! DAP Cell-capturing named-routine assignment mapping and invalidation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str =
    include_str!("../../../tests/debugger/fixtures/cell_capturing_routine_assignment.fpas");

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile cell-capturing fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

fn send(adapter: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    adapter.handle(request(*seq, command, arguments))
}

fn frame(adapter: &mut DapServer, seq: &mut u64) -> u64 {
    send(adapter, seq, "stackTrace", json!({"threadId":1}))[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("frame ID")
}

fn locals_reference(adapter: &mut DapServer, seq: &mut u64, frame_id: u64) -> u64 {
    send(adapter, seq, "scopes", json!({"frameId":frame_id}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variablesReference"].as_u64())
        .expect("locals")
}

fn stop_at(adapter: &mut DapServer, seq: &mut u64, needle: &str) -> u64 {
    let line = SOURCE
        .lines()
        .position(|line| line.contains(needle))
        .expect("marker")
        + 1;
    let _ = send(
        adapter,
        seq,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[{"line":line}]}),
    );
    let _ = send(adapter, seq, "continue", json!({"threadId":1}));
    let _ = adapter.wait();
    frame(adapter, seq)
}

#[test]
fn dap_cell_capturing_assignment_invalidates_and_continues() {
    let mut adapter = server();
    let mut seq = 0;
    let initialized = send(
        &mut adapter,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent":true}),
    );
    assert_eq!(initialized[0]["body"]["supportsSetVariable"], true);
    assert_eq!(initialized[0]["body"]["supportsSetExpression"], true);
    let _ = send(
        &mut adapter,
        &mut seq,
        "launch",
        json!({"stopOnEntry":true}),
    );
    let _ = send(&mut adapter, &mut seq, "configurationDone", json!({}));
    let owner = stop_at(&mut adapter, &mut seq, "var CellStop: integer := 0;");
    let locals = locals_reference(&mut adapter, &mut seq, owner);
    let simple = send(
        &mut adapter,
        &mut seq,
        "setVariable",
        json!({"variablesReference":locals,"name":"Current","value":"AddCell"}),
    );
    assert_eq!(simple[0]["success"], true, "{simple:?}");
    assert_eq!(simple[0]["body"]["value"], "<function mutating.addcell>");

    let current_frame = frame(&mut adapter, &mut seq);
    let assigned = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current_frame,"expression":"Current","value":"Mutating.AddCell"}),
    );
    assert_eq!(assigned[0]["success"], true, "{assigned:?}");
    assert_eq!(assigned[0]["body"]["value"], "<function mutating.addcell>");
    assert!(
        assigned
            .iter()
            .any(|record| record["event"] == "invalidated"),
        "{assigned:?}"
    );

    let current_frame = frame(&mut adapter, &mut seq);
    let copied = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current_frame,"expression":"Copy","value":"Current"}),
    );
    assert_eq!(copied[0]["success"], true, "{copied:?}");
    assert_eq!(copied[0]["body"]["value"], "<function mutating.addcell>");
    assert!(
        copied.iter().any(|record| record["event"] == "invalidated"),
        "{copied:?}"
    );

    let _ = send(&mut adapter, &mut seq, "continue", json!({"threadId":1}));
    let output = adapter
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["output"].as_str())
        .collect::<String>();
    assert!(
        output.starts_with("12\n13\n"),
        "continuation must observe shared cell writes, got {output:?}"
    );
}

#[test]
fn dap_global_cell_destination_is_rejected_without_invalidation() {
    let mut adapter = server();
    let mut seq = 0;
    let _ = send(
        &mut adapter,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent":true}),
    );
    let _ = send(
        &mut adapter,
        &mut seq,
        "launch",
        json!({"stopOnEntry":true}),
    );
    let _ = send(&mut adapter, &mut seq, "configurationDone", json!({}));
    let owner = stop_at(&mut adapter, &mut seq, "var CellStop: integer := 0;");
    let rejected = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":owner,"expression":"Shared","value":"AddCell"}),
    );
    assert_eq!(rejected[0]["success"], false, "{rejected:?}");
    assert!(rejected.iter().all(|record| record.get("event").is_none()));
}
