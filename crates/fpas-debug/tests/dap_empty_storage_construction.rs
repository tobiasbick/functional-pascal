//! DAP seeded empty-storage initialization mapping coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str =
    include_str!("../../../tests/debugger/fixtures/empty_storage_construction.fpas");

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable =
        fpas_compiler::compile(&program).expect("compile DAP empty-storage construction fixture");
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
    send(adapter, seq, "stackTrace", json!({"threadId": 1}))[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("frame ID")
}

fn initialize(adapter: &mut DapServer, invalidation: bool) -> u64 {
    let mut seq = 0;
    let _ = send(
        adapter,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent": invalidation}),
    );
    let _ = send(adapter, &mut seq, "launch", json!({"stopOnEntry": true}));
    let _ = send(adapter, &mut seq, "configurationDone", json!({}));
    seq
}

fn stop_with_empty(adapter: &mut DapServer, seq: &mut u64, name: &str) -> u64 {
    for _ in 0..64 {
        let current = frame(adapter, seq);
        let scopes = send(adapter, seq, "scopes", json!({"frameId": current}));
        let Some(locals) = scopes[0]["body"]["scopes"]
            .as_array()
            .expect("scopes")
            .iter()
            .find(|scope| scope["name"] == "Locals")
            .and_then(|scope| scope["variablesReference"].as_u64())
        else {
            let _ = send(adapter, seq, "stepIn", json!({"threadId": 1}));
            let _ = adapter.wait();
            continue;
        };
        let variables = send(
            adapter,
            seq,
            "variables",
            json!({"variablesReference": locals, "start": 0, "count": 20}),
        );
        let value = variables[0]["body"]["variables"]
            .as_array()
            .expect("variables")
            .iter()
            .find(|variable| variable["name"] == name)
            .and_then(|variable| variable["value"].as_str());
        match value {
            Some("<uninitialized>") => return current,
            Some(other) => panic!("{name} already initialized as {other}"),
            None => {
                let _ = send(adapter, seq, "stepIn", json!({"threadId": 1}));
                let _ = adapter.wait();
            }
        }
    }
    panic!("{name} never became visible uninitialized")
}

#[test]
fn dap_initialize_storage_maps_jsonl_and_invalidates_variables() {
    let mut adapter = server();
    let mut seq = initialize(&mut adapter, true);
    let current = stop_with_empty(&mut adapter, &mut seq, "State");
    let constructed = send(
        &mut adapter,
        &mut seq,
        "fpas/initializeStorage",
        json!({
            "frameId": current,
            "target": "State.Nested.X",
            "initializer": "MakeInitialState()",
            "expression": "42"
        }),
    );
    assert_eq!(constructed[0]["success"], true, "{constructed:?}");
    assert_eq!(constructed[0]["body"]["root"], "State");
    assert_eq!(constructed[0]["body"]["target"], "State.Nested.X");
    assert_eq!(constructed[0]["body"]["value"], "42");
    assert_eq!(constructed[0]["body"]["type"], "integer");
    assert_eq!(constructed.len(), 2, "{constructed:?}");
    assert_eq!(constructed[1]["event"], "invalidated");
    assert_eq!(constructed[1]["body"]["areas"][0], "variables");
}

#[test]
fn dap_initialize_storage_omits_invalidation_on_failure_and_without_capability() {
    let mut adapter = server();
    let mut seq = initialize(&mut adapter, true);
    let current = stop_with_empty(&mut adapter, &mut seq, "State");
    let rejected = send(
        &mut adapter,
        &mut seq,
        "fpas/initializeStorage",
        json!({
            "frameId": current,
            "target": "State",
            "initializer": "MakeInitialState()",
            "expression": "MakeInitialState()"
        }),
    );
    assert_eq!(rejected[0]["success"], false);
    assert_eq!(
        rejected[0]["body"]["error"]["code"],
        "variable_path_unsupported"
    );
    assert!(rejected.iter().all(|record| record.get("event").is_none()));

    let extra = send(
        &mut adapter,
        &mut seq,
        "fpas/initializeStorage",
        json!({
            "frameId": current,
            "target": "State.Count",
            "initializer": "MakeInitialState()",
            "expression": "1",
            "extra": true
        }),
    );
    assert_eq!(extra[0]["success"], false);
    assert_eq!(extra[0]["body"]["error"]["code"], "invalid_request");
    assert!(extra.iter().all(|record| record.get("event").is_none()));

    let mut quiet = server();
    let mut quiet_seq = initialize(&mut quiet, false);
    let quiet_frame = stop_with_empty(&mut quiet, &mut quiet_seq, "State");
    let constructed = send(
        &mut quiet,
        &mut quiet_seq,
        "fpas/initializeStorage",
        json!({
            "frameId": quiet_frame,
            "target": "State.Count",
            "initializer": "MakeInitialState()",
            "expression": "42"
        }),
    );
    assert_eq!(constructed[0]["success"], true, "{constructed:?}");
    assert_eq!(constructed.len(), 1, "{constructed:?}");
}
