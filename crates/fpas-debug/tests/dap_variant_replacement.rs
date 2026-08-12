//! DAP standard complete-value variant replacement mapping and invalidation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/variant_replacement.fpas");

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP variant fixture");
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

fn stop_with_initialized_locals(adapter: &mut DapServer, seq: &mut u64) -> u64 {
    for _ in 0..64 {
        let current = frame(adapter, seq);
        let mut ready = send(
            adapter,
            seq,
            "evaluate",
            json!({"frameId":current,"expression":"StopMarker"}),
        );
        if ready.is_empty() {
            ready = adapter.wait();
        }
        if ready[0]["success"] == true {
            return current;
        }
        let step = send(adapter, seq, "stepIn", json!({"threadId":1}));
        assert_eq!(step[0]["success"], true, "{step:?}");
        let _ = adapter.wait();
    }
    panic!("DAP variant fixture locals never became initialized")
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
    let body = initialized[0]["body"].as_object().expect("initialize body");
    assert!(
        body.keys()
            .all(|key| key != "supportsVariantReplacement"
                && !key.to_lowercase().contains("variant")),
        "no custom variant capability: {body:?}"
    );
    let _ = send(adapter, &mut seq, "launch", json!({"stopOnEntry":true}));
    let _ = send(adapter, &mut seq, "configurationDone", json!({}));
    let current = stop_with_initialized_locals(adapter, &mut seq);
    (seq, current)
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

#[test]
fn dap_set_variable_and_set_expression_replace_complete_variants() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, true);

    let textual = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":initial_frame,"expression":"Selected","value":"Choice.Pair(10, 20)"}),
    );
    assert_eq!(textual.len(), 2, "{textual:?}");
    assert_eq!(textual[0]["success"], true);
    assert_eq!(textual[0]["body"]["value"], "Choice.Pair");
    assert_eq!(textual[1]["event"], "invalidated");
    assert_eq!(textual[1]["body"]["areas"][0], "variables");

    let current = frame(&mut adapter, &mut seq);
    let locals = locals_reference(&mut adapter, &mut seq, current);
    let handle = send(
        &mut adapter,
        &mut seq,
        "setVariable",
        json!({"variablesReference":locals,"name":"PairValue","value":"Choice.Empty"}),
    );
    assert_eq!(handle[0]["success"], true, "{handle:?}");
    assert_eq!(handle[0]["body"]["value"], "Choice.Empty");
    assert_eq!(handle[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let outcome = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Outcome","value":"Error('fail')"}),
    );
    assert_eq!(outcome[0]["body"]["value"], "Error(...)", "{outcome:?}");
    assert_eq!(outcome[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let optional = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Optional","value":"None"}),
    );
    assert_eq!(optional[0]["body"]["value"], "None", "{optional:?}");
    assert_eq!(optional[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let rejected = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Selected","value":"Pair(1, 2)"}),
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(rejected[0]["success"], false);
    assert!(rejected.iter().all(|record| record.get("event").is_none()));

    let current = frame(&mut adapter, &mut seq);
    let wrong_argument = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Selected","value":"Choice.Count('wrong')"}),
    );
    assert_eq!(wrong_argument.len(), 1, "{wrong_argument:?}");
    assert_eq!(wrong_argument[0]["success"], false);
    assert!(
        wrong_argument
            .iter()
            .all(|record| record.get("event").is_none())
    );

    let current = frame(&mut adapter, &mut seq);
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"EmptyValue","value":"Choice.Count(4)"}),
    );
    let current = frame(&mut adapter, &mut seq);
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Failed","value":"Ok(9)"}),
    );
    let current = frame(&mut adapter, &mut seq);
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Missing","value":"Some(8)"}),
    );
    let current = frame(&mut adapter, &mut seq);
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Packed","value":"Error('pack')"}),
    );
    let current = frame(&mut adapter, &mut seq);
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"NestedResult","value":"None"}),
    );
    let current = frame(&mut adapter, &mut seq);
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"PackedHolder.Item","value":"Choice.Pair(1, 2)"}),
    );
    let current = frame(&mut adapter, &mut seq);
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Items[0]","value":"Choice.Empty"}),
    );
    let current = frame(&mut adapter, &mut seq);
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Scores['blue']","value":"Choice.Count(16)"}),
    );
    let _ = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"expression":"GlobalChoice","value":"Choice.Pair(7, 8)"}),
    );

    let _ = send(&mut adapter, &mut seq, "continue", json!({"threadId":1}));
    let output = adapter
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["output"].as_str())
        .collect::<String>();
    assert_eq!(
        output,
        "30\n0\n4\n9\nfail\n9\n0\n8\npack\n0\n3\n0\n16\n15\n99\n"
    );
}

#[test]
fn dap_omits_variant_invalidation_without_client_support() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, false);
    let records = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":initial_frame,"expression":"Selected","value":"Choice.Pair(10, 20)"}),
    );
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["success"], true);
    assert!(records.iter().all(|record| record.get("event").is_none()));
}
