//! DAP custom sequence mutation mapping and invalidation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program DapSequenceMutation;

begin
  mutable var Numbers: array of integer := [1, 2];
  mutable var Text: string := 'A😀B';
  var Marker: integer := Numbers[0]
end.
"#;

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP sequence fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn send(adapter: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    adapter.handle(json!({"seq":*seq,"type":"request","command":command,"arguments":arguments}))
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
            json!({"frameId":current,"expression":"Text"}),
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
    panic!("DAP sequence fixture locals never became initialized")
}

#[test]
fn dap_sequence_requests_map_results_and_invalidation() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, true);
    let inserted = send(
        &mut adapter,
        &mut seq,
        "fpas/arrayInsert",
        json!({"frameId":initial_frame,"target":"Numbers","index":"1","value":"9"}),
    );
    assert_eq!(inserted[0]["body"]["value"], "[3 items]", "{inserted:?}");
    assert_eq!(inserted[0]["body"]["index"], 1);
    assert_eq!(inserted[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let removed = send(
        &mut adapter,
        &mut seq,
        "fpas/arrayRemove",
        json!({"frameId":current,"target":"Numbers","index":"0"}),
    );
    assert_eq!(removed[0]["body"]["removed"], "1", "{removed:?}");
    assert_eq!(removed[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let replaced = send(
        &mut adapter,
        &mut seq,
        "fpas/stringReplaceCharacter",
        json!({"frameId":current,"target":"Text","index":"1","value":"'é'"}),
    );
    assert_eq!(replaced[0]["body"]["value"], "'AéB'", "{replaced:?}");
    assert_eq!(replaced[0]["body"]["oldCharacter"], "'😀'");
    assert_eq!(replaced[0]["body"]["newCharacter"], "'é'");
    assert_eq!(replaced[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let failed = send(
        &mut adapter,
        &mut seq,
        "fpas/arrayRemove",
        json!({"frameId":current,"target":"Numbers","index":"9"}),
    );
    assert_eq!(failed.len(), 1, "{failed:?}");
    assert_eq!(failed[0]["success"], false);
}

#[test]
fn dap_sequence_success_omits_unnegotiated_invalidation() {
    let mut adapter = server();
    let (mut seq, current) = initialize_and_stop(&mut adapter, false);
    let records = send(
        &mut adapter,
        &mut seq,
        "fpas/arrayInsert",
        json!({"frameId":current,"target":"Numbers","index":"2","value":"3"}),
    );
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["success"], true);
}
