//! DAP variant discovery and construction mapping coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/variant_construction.fpas");

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable =
        fpas_compiler::compile(&program).expect("compile DAP variant construction fixture");
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

fn stop_with_initialized_locals(adapter: &mut DapServer, seq: &mut u64) -> u64 {
    for _ in 0..64 {
        let current = frame(adapter, seq);
        let mut ready = send(
            adapter,
            seq,
            "evaluate",
            json!({"frameId": current, "expression": "StopMarker"}),
        );
        if ready.is_empty() {
            ready = adapter.wait();
        }
        if ready[0]["success"] == true {
            return current;
        }
        let _ = send(adapter, seq, "stepIn", json!({"threadId": 1}));
        let _ = adapter.wait();
    }
    panic!("DAP variant construction fixture locals never became initialized")
}

#[test]
fn dap_variant_describe_and_construct_map_jsonl_and_invalidate_variables() {
    let mut adapter = server();
    let mut seq = initialize(&mut adapter, true);
    let current = stop_with_initialized_locals(&mut adapter, &mut seq);
    let described = send(
        &mut adapter,
        &mut seq,
        "fpas/variantDescribe",
        json!({"frameId": current, "target": "Selected"}),
    );
    assert_eq!(described[0]["success"], true, "{described:?}");
    assert_eq!(
        described.len(),
        1,
        "discovery does not invalidate: {described:?}"
    );
    assert_eq!(described[0]["body"]["typeName"], "Choice");
    assert_eq!(described[0]["body"]["variants"][2]["name"], "Choice.Pair");
    assert_eq!(
        described[0]["body"]["variants"][2]["fields"][0]["typeName"],
        "Integer"
    );

    let constructed = send(
        &mut adapter,
        &mut seq,
        "fpas/variantConstruct",
        json!({
            "frameId": current,
            "target": "Selected",
            "variant": "Choice.Pair",
            "fields": {"Left": "1", "Right": "2"}
        }),
    );
    assert_eq!(constructed[0]["success"], true, "{constructed:?}");
    assert_eq!(constructed[0]["body"]["variant"], "Choice.Pair");
    assert_eq!(constructed[0]["body"]["value"], "Choice.Pair");
    assert_eq!(constructed.len(), 2, "{constructed:?}");
    assert_eq!(constructed[1]["event"], "invalidated");
    assert_eq!(constructed[1]["body"]["areas"][0], "variables");
}

#[test]
fn dap_variant_construct_omits_invalidation_on_failure_and_without_capability() {
    let mut adapter = server();
    let mut seq = initialize(&mut adapter, true);
    let current = stop_with_initialized_locals(&mut adapter, &mut seq);
    let rejected = send(
        &mut adapter,
        &mut seq,
        "fpas/variantConstruct",
        json!({
            "frameId": current,
            "target": "Selected",
            "variant": "Choice.Nope",
            "fields": {}
        }),
    );
    assert_eq!(rejected[0]["success"], false);
    assert_eq!(rejected[0]["body"]["error"]["code"], "variant_unknown");
    assert!(rejected.iter().all(|record| record.get("event").is_none()));

    let mut quiet = server();
    let mut quiet_seq = initialize(&mut quiet, false);
    let quiet_frame = stop_with_initialized_locals(&mut quiet, &mut quiet_seq);
    let constructed = send(
        &mut quiet,
        &mut quiet_seq,
        "fpas/variantConstruct",
        json!({
            "frameId": quiet_frame,
            "target": "Selected",
            "variant": "Choice.Empty",
            "fields": {}
        }),
    );
    assert_eq!(constructed[0]["success"], true, "{constructed:?}");
    assert_eq!(constructed.len(), 1, "{constructed:?}");
}
