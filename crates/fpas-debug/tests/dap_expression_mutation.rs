//! DAP `setExpression` capability, argument mapping, and invalidation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program DapExpressionMutation;

type
  Box = record
    Value: integer;
  end;

mutable var
  GlobalValue: integer := 5;

begin
  mutable var Index: integer := 0;
  mutable var Counter: integer := 1;
  mutable var Items: array of Box := [record
    Value := 10;
  end];
  Counter := Counter + 1;
  GlobalValue := GlobalValue + Items[Index].Value
end.
"#;

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP expression fixture");
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
    for _ in 0..24 {
        let current = frame(adapter, seq);
        let mut ready = send(
            adapter,
            seq,
            "evaluate",
            json!({"frameId":current,"expression":"Items"}),
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
    panic!("DAP fixture locals never became initialized")
}

fn initialize_and_stop(adapter: &mut DapServer, invalidation: bool) -> (u64, u64) {
    let mut seq = 0;
    let initialized = send(
        adapter,
        &mut seq,
        "initialize",
        json!({"supportsInvalidatedEvent":invalidation}),
    );
    assert_eq!(initialized[0]["body"]["supportsSetExpression"], true);
    let _ = send(adapter, &mut seq, "launch", json!({"stopOnEntry":true}));
    let _ = send(adapter, &mut seq, "configurationDone", json!({}));
    let current = stop_with_initialized_locals(adapter, &mut seq);
    (seq, current)
}

#[test]
fn dap_set_expression_maps_targets_and_orders_negotiated_invalidation() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, true);

    let local = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":initial_frame,"expression":"Counter","value":"21 * 2"}),
    );
    assert_eq!(local.len(), 2, "{local:?}");
    assert_eq!(local[0]["type"], "response");
    assert_eq!(local[0]["success"], true);
    assert_eq!(local[0]["body"]["value"], "42");
    assert_eq!(local[0]["body"]["type"], "integer");
    assert_eq!(local[1]["event"], "invalidated");
    assert_eq!(local[1]["body"]["areas"][0], "variables");

    let stale = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":initial_frame,"expression":"Counter","value":"1"}),
    );
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0]["success"], false);
    assert!(
        stale[0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("unknown or expired"))
    );

    let current = frame(&mut adapter, &mut seq);
    let nested = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({
            "frameId":current,
            "expression":"Items[Index].Value",
            "value":"43"
        }),
    );
    assert_eq!(nested[0]["body"]["value"], "43", "{nested:?}");
    assert_eq!(nested[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let rejected = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Items[9].Value","value":"1"}),
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(rejected[0]["success"], false);
    assert!(rejected.iter().all(|record| record.get("event").is_none()));

    let mut still_usable = send(
        &mut adapter,
        &mut seq,
        "evaluate",
        json!({"frameId":current,"expression":"Counter"}),
    );
    if still_usable.is_empty() {
        still_usable = adapter.wait();
    }
    assert_eq!(still_usable[0]["body"]["result"], "42");

    let unsupported_format = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({
            "frameId":current,
            "expression":"Counter",
            "value":"1",
            "format":{"hex":true}
        }),
    );
    assert_eq!(unsupported_format.len(), 1);
    assert_eq!(unsupported_format[0]["success"], false);

    let global = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"expression":"GlobalValue","value":"99"}),
    );
    assert_eq!(global[0]["body"]["value"], "99", "{global:?}");
    assert_eq!(global[1]["event"], "invalidated");
}

#[test]
fn dap_set_expression_omits_invalidation_when_client_did_not_negotiate_it() {
    let mut adapter = server();
    let (mut seq, current) = initialize_and_stop(&mut adapter, false);

    let records = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Counter","value":"7"}),
    );

    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["success"], true);
    assert_eq!(records[0]["body"]["value"], "7");
}
