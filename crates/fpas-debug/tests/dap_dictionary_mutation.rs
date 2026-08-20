//! DAP custom dictionary mutation mapping and invalidation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program DapDictionaryMutation;

begin
  mutable var Scores: dict of string to integer := ['Ada': 1, 'Grace': 2];
  var Marker: integer := Scores['Grace']
end.
"#;

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP dictionary fixture");
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

fn initialize_and_stop(adapter: &mut DapServer, invalidation: bool) -> (u64, u64) {
    let mut seq = 0;
    let _ = send(
        adapter,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent":invalidation}),
    );
    let _ = send(adapter, &mut seq, "launch", json!({"stopOnEntry":true}));
    let _ = send(adapter, &mut seq, "configurationDone", json!({}));
    for _ in 0..16 {
        let current = frame(adapter, &mut seq);
        let mut ready = send(
            adapter,
            &mut seq,
            "evaluate",
            json!({"frameId":current,"expression":"Scores"}),
        );
        if ready.is_empty() {
            ready = adapter.wait();
        }
        if ready[0]["success"] == true {
            return (seq, current);
        }
        let _ = send(adapter, &mut seq, "stepIn", json!({"threadId":1}));
        let _ = adapter.wait();
    }
    panic!("DAP dictionary fixture locals never became initialized")
}

#[test]
fn dap_dictionary_requests_map_results_and_order_invalidation() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, true);

    let inserted = send(
        &mut adapter,
        &mut seq,
        "fpas/dictionaryInsert",
        json!({"frameId":initial_frame,"target":"Scores","key":"'Bob'","value":"3"}),
    );
    assert_eq!(inserted.len(), 2, "{inserted:?}");
    assert_eq!(inserted[0]["success"], true);
    assert_eq!(inserted[0]["body"]["value"], "{3 entries}");
    assert_eq!(inserted[0]["body"]["type"], "dict");
    assert_eq!(inserted[0]["body"]["namedVariables"], 6);
    assert_eq!(inserted[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let removed = send(
        &mut adapter,
        &mut seq,
        "fpas/dictionaryRemove",
        json!({"frameId":current,"target":"Scores","key":"'Ada'"}),
    );
    assert_eq!(removed[0]["body"]["removed"], "1", "{removed:?}");
    assert_eq!(removed[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let replaced = send(
        &mut adapter,
        &mut seq,
        "fpas/dictionaryReplaceKey",
        json!({"frameId":current,"target":"Scores","key":"'Grace'","newKey":"'Hopper'"}),
    );
    assert_eq!(replaced[0]["body"]["oldKey"], "'Grace'", "{replaced:?}");
    assert_eq!(replaced[0]["body"]["newKey"], "'Hopper'");
    assert_eq!(replaced[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let failed = send(
        &mut adapter,
        &mut seq,
        "fpas/dictionaryRemove",
        json!({"frameId":current,"target":"Scores","key":"'Missing'"}),
    );
    assert_eq!(failed.len(), 1, "{failed:?}");
    assert_eq!(failed[0]["success"], false);
    assert!(failed.iter().all(|record| record.get("event").is_none()));

    let mut usable = send(
        &mut adapter,
        &mut seq,
        "evaluate",
        json!({"frameId":current,"expression":"Scores['Bob']"}),
    );
    if usable.is_empty() {
        usable = adapter.wait();
    }
    assert_eq!(usable[0]["body"]["result"], "3");
}

#[test]
fn dap_dictionary_success_omits_invalidation_when_not_negotiated() {
    let mut adapter = server();
    let (mut seq, current) = initialize_and_stop(&mut adapter, false);
    let records = send(
        &mut adapter,
        &mut seq,
        "fpas/dictionaryInsert",
        json!({"frameId":current,"target":"Scores","key":"'Bob'","value":"3"}),
    );
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["success"], true);
}
