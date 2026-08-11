//! DAP `setVariable` capability, response shape, and invalidation ordering.

#![allow(
    clippy::expect_used,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

fn server() -> DapServer {
    let source = "program Main;\nbegin\n  mutable var X: integer := 1;\n  X := X + 1\nend.";
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile mutation fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

fn stop_after_initialization(adapter: &mut DapServer) {
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":true})));
    let _ = adapter.handle(request(3, "configurationDone", json!({})));
    let _ = adapter.handle(request(4, "stepIn", json!({"threadId":1})));
    let _ = adapter.wait();
}

fn locals_reference(adapter: &mut DapServer, seq: &mut u64) -> u64 {
    *seq += 1;
    let stack = adapter.handle(request(*seq, "stackTrace", json!({"threadId":1})));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("frame");
    *seq += 1;
    let scopes = adapter.handle(request(*seq, "scopes", json!({"frameId":frame})));
    scopes[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variablesReference"].as_u64())
        .expect("locals")
}

#[test]
fn dap_set_variable_returns_before_client_negotiated_invalidation() {
    let mut adapter = server();
    let initialized = adapter.handle(request(
        1,
        "initialize",
        json!({"supportsInvalidatedEvent":true}),
    ));
    assert_eq!(initialized[0]["body"]["supportsSetVariable"], true);
    assert_eq!(initialized[0]["body"]["supportsSetExpression"], false);
    stop_after_initialization(&mut adapter);

    let mut seq = 4;
    let locals = locals_reference(&mut adapter, &mut seq);
    seq += 1;
    let records = adapter.handle(request(
        seq,
        "setVariable",
        json!({"variablesReference":locals,"name":"X","value":"20 + 2"}),
    ));
    assert_eq!(records.len(), 2, "{records:?}");
    assert_eq!(records[0]["type"], "response");
    assert_eq!(records[0]["success"], true);
    assert_eq!(records[0]["body"]["value"], "22");
    assert_eq!(records[0]["body"]["type"], "integer");
    assert_eq!(records[1]["event"], "invalidated");
    assert_eq!(records[1]["body"]["areas"][0], "variables");

    seq += 1;
    let unsupported_format = adapter.handle(request(
        seq,
        "setVariable",
        json!({
            "variablesReference":locals,
            "name":"X",
            "value":"1",
            "format":{"hex":true}
        }),
    ));
    assert_eq!(unsupported_format.len(), 1);
    assert_eq!(unsupported_format[0]["success"], false);
    assert!(
        unsupported_format
            .iter()
            .all(|record| record.get("event").is_none()),
        "failed mutation must not invalidate variables"
    );
}

#[test]
fn dap_omits_invalidation_for_clients_that_do_not_support_it() {
    let mut adapter = server();
    let _ = adapter.handle(request(1, "initialize", json!({})));
    stop_after_initialization(&mut adapter);
    let mut seq = 4;
    let locals = locals_reference(&mut adapter, &mut seq);
    seq += 1;
    let records = adapter.handle(request(
        seq,
        "setVariable",
        json!({"variablesReference":locals,"name":"X","value":"7"}),
    ));
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["success"], true);
}
