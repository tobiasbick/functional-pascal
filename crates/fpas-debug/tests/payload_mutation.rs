//! JSONL payload-mutation success, nested paths, stable errors, and continuation.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/payload_mutation.fpas");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile payload fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn send(server: &mut JsonlServer, id: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *id += 1;
    server.handle_line(&request(*id, command, arguments))
}

fn frame(server: &mut JsonlServer, id: &mut u64) -> u64 {
    send(server, id, "stack", json!({}))[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("frame ID")
}

fn stop_with_initialized_locals(server: &mut JsonlServer, id: &mut u64) -> u64 {
    for _ in 0..48 {
        let current = frame(server, id);
        let ready = send(
            server,
            id,
            "evaluate",
            json!({"frame_id":current,"expression":"StopMarker"}),
        );
        if ready[0]["success"] == true {
            return current;
        }
        let _ = send(server, id, "step_into", json!({}));
        let _ = server.wait();
    }
    panic!("payload fixture locals never became initialized")
}

fn locals_reference(server: &mut JsonlServer, id: &mut u64, frame_id: u64) -> u64 {
    send(server, id, "scopes", json!({"frame_id":frame_id}))[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("locals")
}

fn child_reference(server: &mut JsonlServer, id: &mut u64, parent: u64, name: &str) -> u64 {
    send(
        server,
        id,
        "variables",
        json!({"variables_reference":parent}),
    )[0]["body"]["variables"]
        .as_array()
        .expect("variables")
        .iter()
        .find(|variable| variable["name"] == name)
        .and_then(|variable| variable["variables_reference"].as_u64())
        .unwrap_or_else(|| panic!("{name} reference"))
}

#[test]
fn jsonl_payload_mutations_commit_atomically_and_continue() {
    let mut server = server();
    let mut id = 0;
    let initialized = send(&mut server, &mut id, "initialize", json!({"version":2}));
    assert_eq!(initialized[0]["body"]["capabilities"]["set_variable"], true);
    assert_eq!(
        initialized[0]["body"]["capabilities"]["set_expression"],
        true
    );
    assert!(
        initialized[0]["body"]["capabilities"]
            .as_object()
            .expect("capabilities")
            .keys()
            .all(|key| key != "payload_set" && !key.contains("enum_payload")),
        "no custom payload capability"
    );
    let _ = send(
        &mut server,
        &mut id,
        "launch",
        json!({"stop_on_entry":true}),
    );
    let initial_frame = stop_with_initialized_locals(&mut server, &mut id);

    let selected = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":initial_frame,"target":"Selected.Value","expression":"10"}),
    );
    assert_eq!(selected[0]["body"]["result"], "10", "{selected:?}");

    let current = frame(&mut server, &mut id);
    let pair = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"pAiRvAlUe.rIgHt","expression":"8"}),
    );
    assert_eq!(pair[0]["body"]["result"], "8", "{pair:?}");

    let current = frame(&mut server, &mut id);
    let nested = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"NestedValue.Item.X","expression":"6"}),
    );
    assert_eq!(nested[0]["body"]["result"], "6", "{nested:?}");

    let current = frame(&mut server, &mut id);
    let locals = locals_reference(&mut server, &mut id, current);
    let outcome = child_reference(&mut server, &mut id, locals, "Outcome");
    let ok = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":outcome,"name":"value","expression":"20"}),
    );
    assert_eq!(ok[0]["body"]["result"], "20", "{ok:?}");

    let current = frame(&mut server, &mut id);
    let failed = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Failed.value","expression":"'new'"}),
    );
    assert_eq!(failed[0]["body"]["result"], "'new'", "{failed:?}");

    let current = frame(&mut server, &mut id);
    let optional = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Optional.value","expression":"70"}),
    );
    assert_eq!(optional[0]["body"]["result"], "70", "{optional:?}");

    let current = frame(&mut server, &mut id);
    let packed_item = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Packed.value.Items[1]","expression":"90"}),
    );
    assert_eq!(packed_item[0]["body"]["result"], "90", "{packed_item:?}");

    let current = frame(&mut server, &mut id);
    let packed_score = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({
            "frame_id":current,
            "target":"Packed.value.Scores['blue']",
            "expression":"40"
        }),
    );
    assert_eq!(packed_score[0]["body"]["result"], "40", "{packed_score:?}");

    let current = frame(&mut server, &mut id);
    let nested_result = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"NestedResult.value.value","expression":"42"}),
    );
    assert_eq!(
        nested_result[0]["body"]["result"], "42",
        "{nested_result:?}"
    );

    let current = frame(&mut server, &mut id);
    for (target, expression, code) in [
        ("Missing.value", "1", "variable_path_unsupported"),
        ("Selected.Left", "1", "variable_path_unsupported"),
        ("Outcome.count", "1", "variable_target_unknown"),
        ("Selected.Value", "'wrong'", "variable_value_type"),
        ("Fixed.Value", "1", "variable_not_mutable"),
    ] {
        let failed = send(
            &mut server,
            &mut id,
            "expression.set",
            json!({"frame_id":current,"target":target,"expression":expression}),
        );
        assert_eq!(failed[0]["error"]["code"], code, "{target}: {failed:?}");
        let preserved = send(
            &mut server,
            &mut id,
            "evaluate",
            json!({"frame_id":current,"expression":"Optional"}),
        );
        assert_eq!(preserved[0]["success"], true, "{preserved:?}");
    }

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "10\n10\n11\n20\nnew\n70\n90\n40\n42\n99\n");
}

#[test]
fn jsonl_payload_requests_validate_lifecycle() {
    let mut created = server();
    let invalid = created.handle_line(&request(
        1,
        "expression.set",
        json!({"target":"Selected.Value","expression":"1"}),
    ));
    assert_eq!(invalid[0]["error"]["code"], "invalid_state");

    let mut running = server();
    let mut id = 0;
    let _ = send(&mut running, &mut id, "initialize", json!({}));
    let _ = send(
        &mut running,
        &mut id,
        "launch",
        json!({"stop_on_entry":false}),
    );
    let invalid = send(
        &mut running,
        &mut id,
        "expression.set",
        json!({"target":"Selected.Value","expression":"1"}),
    );
    assert_eq!(invalid[0]["error"]["code"], "invalid_state");
    let _ = send(&mut running, &mut id, "disconnect", json!({}));
}

#[test]
fn jsonl_payload_mutation_stays_bound_to_the_selected_child_task() {
    const TASK_SOURCE: &str = r#"program TaskPayloadMutation;

uses Std.Console, Std.Task;

function Work(): integer;
begin
  mutable var Optional: Option of integer := Some(1);
  var Marker: integer := 0;
  case Optional of
    Some(Value):
    begin
      return Value
    end;
    None:
    begin
      return 0
    end
  end
end;

begin
  var Pending: task := go Work();
  WriteLn(Wait(Pending))
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(TASK_SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task payload fixture");
    let mut server =
        JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server");
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let breakpoint = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":8}),
    ));
    assert_eq!(breakpoint[0]["body"]["verified"], true, "{breakpoint:?}");
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let stopped = server.wait();
    assert!(
        stopped
            .iter()
            .any(|record| { record["event"] == "stopped" && record["body"]["task_id"] == 1 })
    );

    let child_stack = server.handle_line(&request(4, "stack", json!({"task_id":1})));
    let child_frame = child_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("child frame");
    let main_stack = server.handle_line(&request(5, "stack", json!({"task_id":0})));
    let main_frame = main_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("main frame");

    let updated = server.handle_line(&request(
        6,
        "expression.set",
        json!({"frame_id":child_frame,"target":"Optional.value","expression":"5"}),
    ));
    assert_eq!(updated[0]["body"]["result"], "5", "{updated:?}");
    let expired_main = server.handle_line(&request(7, "scopes", json!({"frame_id":main_frame})));
    assert_eq!(expired_main[0]["error"]["code"], "unknown_frame");

    let _ = server.handle_line(&request(8, "continue", json!({})));
    let terminated = server.wait();
    assert!(
        terminated
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "5\n" })
    );
}
