//! DAP standard uninitialized-root assignment mapping and invalidation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/uninitialized_assignment.fpas");

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP uninitialized fixture");
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

fn stop_with_uninitialized_count(adapter: &mut DapServer, seq: &mut u64) -> u64 {
    for _ in 0..64 {
        let current = frame(adapter, seq);
        let scopes = send(adapter, seq, "scopes", json!({"frameId":current}));
        let Some(locals) = scopes[0]["body"]["scopes"]
            .as_array()
            .expect("scopes")
            .iter()
            .find(|scope| scope["name"] == "Locals")
            .and_then(|scope| scope["variablesReference"].as_u64())
        else {
            let step = send(adapter, seq, "stepIn", json!({"threadId":1}));
            assert_eq!(step[0]["success"], true, "{step:?}");
            let _ = adapter.wait();
            continue;
        };
        let mut variables = send(
            adapter,
            seq,
            "variables",
            json!({"variablesReference":locals,"start":0,"count":20}),
        );
        if variables.is_empty() {
            variables = adapter.wait();
        }
        let count = variables[0]["body"]["variables"]
            .as_array()
            .expect("locals")
            .iter()
            .find(|value| value["name"] == "Count");
        match count.and_then(|value| value["value"].as_str()) {
            Some("<uninitialized>") => return current,
            Some(other) => panic!("Count already initialized as {other}"),
            None => {
                let step = send(adapter, seq, "stepIn", json!({"threadId":1}));
                assert_eq!(step[0]["success"], true, "{step:?}");
                let _ = adapter.wait();
            }
        }
    }
    panic!("DAP uninitialized Count never became visible")
}

fn initialize_and_stop(adapter: &mut DapServer, invalidation: bool) -> (u64, u64) {
    let mut seq = 0;
    let initialized = send(
        adapter,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent":invalidation}),
    );
    assert_eq!(initialized[0]["body"]["supportsSetVariable"], true);
    assert_eq!(initialized[0]["body"]["supportsSetExpression"], true);
    let _ = send(adapter, &mut seq, "launch", json!({"stopOnEntry":true}));
    let _ = send(adapter, &mut seq, "configurationDone", json!({}));
    let current = stop_with_uninitialized_count(adapter, &mut seq);
    (seq, current)
}

#[test]
fn dap_set_variable_and_set_expression_initialize_uninitialized_roots() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, true);

    let textual = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":initial_frame,"expression":"Count","value":"30"}),
    );
    assert_eq!(textual.len(), 2, "{textual:?}");
    assert_eq!(textual[0]["success"], true);
    assert_eq!(textual[0]["body"]["value"], "30");
    assert_eq!(textual[1]["event"], "invalidated");
    assert_eq!(textual[1]["body"]["areas"][0], "variables");

    let current = frame(&mut adapter, &mut seq);
    let locals = locals_reference(&mut adapter, &mut seq, current);
    let handle = send(
        &mut adapter,
        &mut seq,
        "setVariable",
        json!({"variablesReference":locals,"name":"Flag","value":"true"}),
    );
    assert_eq!(handle[0]["success"], true, "{handle:?}");
    assert_eq!(handle[0]["body"]["value"], "true");
    assert_eq!(handle[1]["event"], "invalidated");

    let global = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"expression":"GlobalCount","value":"8"}),
    );
    assert_eq!(global[0]["success"], true, "{global:?}");
    assert_eq!(global[0]["body"]["value"], "8");
    assert_eq!(global[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let rejected = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Origin.X","value":"1"}),
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(rejected[0]["success"], false);
    assert!(rejected.iter().all(|record| record.get("event").is_none()));

    let _ = send(&mut adapter, &mut seq, "continue", json!({"threadId":1}));
    let output = adapter
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["output"].as_str())
        .collect::<String>();
    assert_eq!(output, "30\n8\n1\n3\n2\n");
}

#[test]
fn dap_omits_initialization_invalidation_without_client_support() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, false);
    let records = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":initial_frame,"expression":"Count","value":"30"}),
    );
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["success"], true);
    assert!(records.iter().all(|record| record.get("event").is_none()));
}
