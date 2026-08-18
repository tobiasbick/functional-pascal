//! DAP function-value assignment mapping and invalidation coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "DAP transcript tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str =
    include_str!("../../../tests/debugger/fixtures/function_value_assignment.fpas");

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile DAP function-value fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn jsonl_server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable =
        fpas_compiler::compile(&program).expect("compile JSONL function-value fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
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
    panic!("DAP function-value fixture locals never became initialized")
}

fn evaluate(adapter: &mut DapServer, seq: &mut u64, frame_id: u64, expression: &str) -> Vec<Value> {
    let mut result = send(
        adapter,
        seq,
        "evaluate",
        json!({"frameId":frame_id,"expression":expression}),
    );
    if result.is_empty() {
        result = adapter.wait();
    }
    result
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

fn variable<'a>(variables: &'a [Value], name: &str) -> &'a Value {
    variables
        .iter()
        .find(|variable| variable["name"] == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn listed_variables(adapter: &mut DapServer, seq: &mut u64, reference: u64) -> Vec<Value> {
    send(
        adapter,
        seq,
        "variables",
        json!({"variablesReference":reference}),
    )[0]["body"]["variables"]
        .as_array()
        .expect("variables")
        .clone()
}

fn child_handle(variables: &[Value], name: &str) -> u64 {
    variable(variables, name)["variablesReference"]
        .as_u64()
        .unwrap_or_else(|| panic!("{name} variablesReference"))
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
    let current = stop_with_initialized_locals(adapter, &mut seq);
    (seq, current)
}

fn jsonl_send(
    server: &mut JsonlServer,
    id: &mut u64,
    command: &str,
    arguments: Value,
) -> Vec<Value> {
    *id += 1;
    server.handle_line(
        &json!({"type":"request","id":*id,"command":command,"arguments":arguments}).to_string(),
    )
}

fn jsonl_frame(server: &mut JsonlServer, id: &mut u64) -> u64 {
    jsonl_send(server, id, "stack", json!({}))[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("JSONL frame ID")
}

fn initialize_and_stop_jsonl(server: &mut JsonlServer) -> (u64, u64) {
    let mut id = 0;
    let _ = jsonl_send(server, &mut id, "initialize", json!({"version":2}));
    let _ = jsonl_send(server, &mut id, "launch", json!({"stop_on_entry":true}));
    for _ in 0..64 {
        let current = jsonl_frame(server, &mut id);
        let ready = jsonl_send(
            server,
            &mut id,
            "evaluate",
            json!({"frame_id":current,"expression":"StopMarker"}),
        );
        if ready[0]["success"] == true {
            return (id, current);
        }
        let _ = jsonl_send(server, &mut id, "step_into", json!({}));
        let _ = server.wait();
    }
    panic!("JSONL function-value fixture locals never became initialized")
}

#[test]
fn dap_set_variable_and_set_expression_copy_function_values() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, true);

    let textual = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":initial_frame,"expression":"Current","value":"Backup"}),
    );
    assert_eq!(textual.len(), 2, "{textual:?}");
    assert_eq!(textual[0]["success"], true);
    assert_eq!(textual[0]["body"]["value"], "<function addtwo>");
    assert_eq!(textual[1]["event"], "invalidated");
    assert_eq!(textual[1]["body"]["areas"][0], "variables");

    let current = frame(&mut adapter, &mut seq);
    let invoked = evaluate(&mut adapter, &mut seq, current, "Current(1)");
    assert_eq!(invoked[0]["success"], true, "{invoked:?}");
    assert_eq!(invoked[0]["body"]["result"], "3");

    let locals = locals_reference(&mut adapter, &mut seq, current);
    let handle = send(
        &mut adapter,
        &mut seq,
        "setVariable",
        json!({"variablesReference":locals,"name":"Current","value":"Captured"}),
    );
    assert_eq!(handle[0]["success"], true, "{handle:?}");
    assert_eq!(handle[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let rejected = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Frozen","value":"Backup"}),
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(rejected[0]["success"], false);
    assert!(rejected.iter().all(|record| record.get("event").is_none()));

    let named = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Current","value":"AddTwo"}),
    );
    assert_eq!(named.len(), 2, "{named:?}");
    assert_eq!(named[0]["success"], true, "{named:?}");
    assert_eq!(named[0]["body"]["value"], "<function addtwo>");
    assert_eq!(named[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let named_call = evaluate(&mut adapter, &mut seq, current, "Current(1)");
    assert_eq!(named_call[0]["body"]["result"], "3");

    let qualified = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Current","value":"Math.Transform"}),
    );
    assert_eq!(qualified[0]["success"], true, "{qualified:?}");
    assert_eq!(qualified[1]["event"], "invalidated");

    let current = frame(&mut adapter, &mut seq);
    let bound = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Current","value":"Receiver.Add"}),
    );
    assert_eq!(bound[0]["success"], true, "{bound:?}");
    assert_eq!(bound[0]["body"]["value"], "<function Counter.Add>");
    assert_eq!(bound[1]["event"], "invalidated");
    let current = frame(&mut adapter, &mut seq);
    let bound_call = evaluate(&mut adapter, &mut seq, current, "Current(1)");
    assert_eq!(bound_call[0]["body"]["result"], "11", "{bound_call:?}");

    let reset = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Current","value":"Math.Transform"}),
    );
    assert_eq!(reset[0]["success"], true, "{reset:?}");
    let current = frame(&mut adapter, &mut seq);
    let wrong_signature = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"WrongSignature","value":"Receiver.Add"}),
    );
    assert_eq!(wrong_signature.len(), 1, "{wrong_signature:?}");
    assert_eq!(wrong_signature[0]["success"], false);
    assert!(
        wrong_signature
            .iter()
            .all(|record| record.get("event").is_none())
    );

    let ambiguous = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Current","value":"Transform"}),
    );
    assert_eq!(ambiguous.len(), 1, "{ambiguous:?}");
    assert_eq!(ambiguous[0]["success"], false);
    assert!(ambiguous.iter().all(|record| record.get("event").is_none()));

    let _ = send(&mut adapter, &mut seq, "continue", json!({"threadId":1}));
    let output = adapter
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["output"].as_str())
        .collect::<String>();
    assert_eq!(output, "4\n11\n");
}

#[test]
fn dap_omits_function_assignment_invalidation_without_client_support() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, false);
    let records = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":initial_frame,"expression":"Current","value":"Backup"}),
    );
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["success"], true);
    assert!(records.iter().all(|record| record.get("event").is_none()));
}

#[test]
fn dap_and_jsonl_function_assignment_results_and_errors_match() {
    let mut dap = server();
    let (mut dap_seq, dap_frame) = initialize_and_stop(&mut dap, false);
    let mut jsonl = jsonl_server();
    let (mut jsonl_id, initial_jsonl_frame) = initialize_and_stop_jsonl(&mut jsonl);

    let dap_success = send(
        &mut dap,
        &mut dap_seq,
        "setExpression",
        json!({"frameId":dap_frame,"expression":"Current","value":"Backup"}),
    );
    let jsonl_success = jsonl_send(
        &mut jsonl,
        &mut jsonl_id,
        "expression.set",
        json!({"frame_id":initial_jsonl_frame,"target":"Current","expression":"Backup"}),
    );
    assert_eq!(
        dap_success[0]["body"]["value"],
        jsonl_success[0]["body"]["result"]
    );
    assert_eq!(
        dap_success[0]["body"]["type"],
        jsonl_success[0]["body"]["type_name"]
    );

    let dap_frame = frame(&mut dap, &mut dap_seq);
    let current_jsonl_frame = jsonl_frame(&mut jsonl, &mut jsonl_id);
    let dap_failure = send(
        &mut dap,
        &mut dap_seq,
        "setExpression",
        json!({"frameId":dap_frame,"expression":"Current","value":"MakeAdder(1)"}),
    );
    let jsonl_failure = jsonl_send(
        &mut jsonl,
        &mut jsonl_id,
        "expression.set",
        json!({"frame_id":current_jsonl_frame,"target":"Current","expression":"MakeAdder(1)"}),
    );
    assert_eq!(
        dap_failure[0]["body"]["error"]["code"],
        jsonl_failure[0]["error"]["code"]
    );
    assert_eq!(
        dap_failure[0]["body"]["error"]["format"],
        jsonl_failure[0]["error"]["message"]
    );
    assert_eq!(
        dap_failure[0]["body"]["error"]["help"],
        jsonl_failure[0]["error"]["help"]
    );

    let closure = "function(Value: integer): integer begin return Value end";
    let dap_closure = send(
        &mut dap,
        &mut dap_seq,
        "setExpression",
        json!({"frameId":dap_frame,"expression":"Current","value":closure}),
    );
    let jsonl_closure = jsonl_send(
        &mut jsonl,
        &mut jsonl_id,
        "expression.set",
        json!({"frame_id":current_jsonl_frame,"target":"Current","expression":closure}),
    );
    assert_eq!(
        dap_closure[0]["body"]["error"]["code"],
        "unsupported_expression"
    );
    assert_eq!(
        dap_closure[0]["body"]["error"]["code"],
        jsonl_closure[0]["error"]["code"]
    );
    assert!(
        dap_closure
            .iter()
            .all(|record| record.get("event").is_none())
    );
}

#[test]
fn dap_synthetic_function_children_stay_non_assignable_without_invalidation() {
    let mut adapter = server();
    let (mut seq, initial_frame) = initialize_and_stop(&mut adapter, true);
    let locals = locals_reference(&mut adapter, &mut seq, initial_frame);
    let locals_page = listed_variables(&mut adapter, &mut seq, locals);
    let captured_handle = child_handle(&locals_page, "Captured");
    assert_ne!(captured_handle, 0);
    let children = listed_variables(&mut adapter, &mut seq, captured_handle);
    assert!(
        children.iter().any(|child| child["name"] == "capture[0]"),
        "{children:?}"
    );

    let rejected = send(
        &mut adapter,
        &mut seq,
        "setVariable",
        json!({"variablesReference":captured_handle,"name":"capture[0]","value":"99"}),
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(rejected[0]["success"], false);
    assert!(rejected.iter().all(|record| record.get("event").is_none()));
    assert_eq!(
        rejected[0]["body"]["error"]["code"],
        "variable_path_unsupported"
    );

    let current = frame(&mut adapter, &mut seq);
    let bound = send(
        &mut adapter,
        &mut seq,
        "setExpression",
        json!({"frameId":current,"expression":"Current","value":"Receiver.Add"}),
    );
    assert_eq!(bound[0]["success"], true, "{bound:?}");
    assert_eq!(bound[1]["event"], "invalidated");
    let current = frame(&mut adapter, &mut seq);
    let locals = locals_reference(&mut adapter, &mut seq, current);
    let locals_page = listed_variables(&mut adapter, &mut seq, locals);
    let receiver_handle = child_handle(&locals_page, "Current");
    let children = listed_variables(&mut adapter, &mut seq, receiver_handle);
    assert!(
        children.iter().any(|child| child["name"] == "receiver"),
        "{children:?}"
    );
    let rejected = send(
        &mut adapter,
        &mut seq,
        "setVariable",
        json!({"variablesReference":receiver_handle,"name":"receiver","value":"Receiver"}),
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert_eq!(rejected[0]["success"], false);
    assert!(rejected.iter().all(|record| record.get("event").is_none()));
    let invoked = evaluate(&mut adapter, &mut seq, current, "Current(1)");
    assert_eq!(invoked[0]["body"]["result"], "11", "{invoked:?}");
}
