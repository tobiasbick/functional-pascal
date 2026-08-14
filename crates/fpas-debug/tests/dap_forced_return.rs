//! DAP forced-return mapping and invalidation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/forced_return.fpas");

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP forced-return fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn jsonl_server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile JSONL forced-return fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

fn send(adapter: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    adapter.handle(request(*seq, command, arguments))
}

fn frames(adapter: &mut DapServer, seq: &mut u64) -> Vec<Value> {
    send(adapter, seq, "stackTrace", json!({"threadId": 1}))[0]["body"]["stackFrames"]
        .as_array()
        .expect("frames")
        .clone()
}

fn stop_in_function(adapter: &mut DapServer, seq: &mut u64, name: &str) -> u64 {
    for _ in 0..64 {
        let current = frames(adapter, seq);
        if current.first().is_some_and(|frame| frame["name"] == name) {
            return current[0]["id"].as_u64().expect("frame ID");
        }
        let step = send(adapter, seq, "stepIn", json!({"threadId": 1}));
        assert_eq!(step[0]["success"], true, "{step:?}");
        let _ = adapter.wait();
    }
    panic!("{name} never became the active callee")
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

#[test]
fn dap_force_return_matches_jsonl_and_invalidates_stacks_and_variables() {
    let mut adapter = server();
    let mut seq = initialize(&mut adapter, true);
    let callee = stop_in_function(&mut adapter, &mut seq, "compute");
    let returned = send(
        &mut adapter,
        &mut seq,
        "fpas/forceReturn",
        json!({"frameId": callee, "expression": "99"}),
    );
    assert_eq!(returned[0]["success"], true, "{returned:?}");
    assert_eq!(returned[0]["body"]["value"], "99");
    assert_eq!(returned[0]["body"]["type"], "integer");
    assert_eq!(returned[0]["body"]["unwoundFrames"], 1);
    assert_eq!(returned[0]["body"]["frame"]["name"], "forcedreturn");
    assert_eq!(returned.len(), 2, "{returned:?}");
    assert_eq!(returned[1]["event"], "invalidated");
    assert_eq!(returned[1]["body"]["areas"][0], "stacks");
    assert_eq!(returned[1]["body"]["areas"][1], "variables");

    let mut jsonl = jsonl_server();
    let mut id = 0u64;
    id += 1;
    let _ = jsonl.handle_line(
        &json!({"type":"request","id":id,"command":"initialize","arguments":{"version":2}})
            .to_string(),
    );
    id += 1;
    let _ = jsonl.handle_line(
        &json!({"type":"request","id":id,"command":"launch","arguments":{"stop_on_entry":true}})
            .to_string(),
    );
    let jsonl_callee = {
        let mut found = None;
        for _ in 0..64 {
            id += 1;
            let frames = jsonl.handle_line(
                &json!({"type":"request","id":id,"command":"stack","arguments":{}}).to_string(),
            );
            if frames[0]["body"]["frames"][0]["name"] == "compute" {
                found = frames[0]["body"]["frames"][0]["frame_id"].as_u64();
                break;
            }
            id += 1;
            let _ = jsonl.handle_line(
                &json!({"type":"request","id":id,"command":"step_into","arguments":{}}).to_string(),
            );
            let _ = jsonl.wait();
        }
        found.expect("JSONL compute frame")
    };
    id += 1;
    let jsonl_result = jsonl.handle_line(
        &json!({"type":"request","id":id,"command":"frame.return","arguments":{"frame_id":jsonl_callee,"expression":"99"}}).to_string(),
    );
    assert_eq!(
        jsonl_result[0]["body"]["result"],
        returned[0]["body"]["value"]
    );
    assert_eq!(
        jsonl_result[0]["body"]["unwound_frames"],
        returned[0]["body"]["unwoundFrames"]
    );

    let rejected = send(
        &mut adapter,
        &mut seq,
        "fpas/forceReturn",
        json!({"frameId": callee, "expression": "1"}),
    );
    assert_eq!(rejected[0]["success"], false);
    assert_eq!(rejected[0]["body"]["error"]["code"], "unknown_frame");
    assert!(rejected.iter().all(|record| record.get("event").is_none()));
}

#[test]
fn dap_force_return_omits_invalidation_when_the_function_requires_a_value() {
    let mut adapter = server();
    let mut seq = initialize(&mut adapter, true);
    let callee = stop_in_function(&mut adapter, &mut seq, "compute");
    let rejected = send(
        &mut adapter,
        &mut seq,
        "fpas/forceReturn",
        json!({"frameId": callee}),
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(
        rejected[0]["body"]["error"]["code"],
        "frame_return_value_required"
    );
}

#[test]
fn dap_force_return_unwinds_a_selected_older_frame_and_invalidates_once() {
    let mut adapter = server();
    let mut seq = initialize(&mut adapter, true);
    let _leaf = stop_in_function(&mut adapter, &mut seq, "leaf");
    let branch = frames(&mut adapter, &mut seq)[1]["id"]
        .as_u64()
        .expect("selected frame");
    let returned = send(
        &mut adapter,
        &mut seq,
        "fpas/forceReturn",
        json!({"frameId": branch, "expression": "Local"}),
    );
    assert_eq!(returned[0]["success"], true, "{returned:?}");
    assert_eq!(returned[0]["body"]["value"], "11");
    assert_eq!(returned[0]["body"]["unwoundFrames"], 2);
    assert_eq!(returned[0]["body"]["frame"]["name"], "forcedreturn");
    assert_eq!(returned.len(), 2, "{returned:?}");
    assert_eq!(returned[1]["event"], "invalidated");
    assert_eq!(returned[1]["body"]["areas"][0], "stacks");
    assert_eq!(returned[1]["body"]["areas"][1], "variables");
}
