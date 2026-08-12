//! JSONL qualified variant-transition assignment coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use fpas_vm::{DebugAssignmentTarget, DebugExpression, DebugRunResult, DebugSession};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/variant_transition.fpas");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile variant transition fixture");
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
    for _ in 0..64 {
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
    panic!("variant transition fixture locals never became initialized")
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

#[test]
fn jsonl_variant_transitions_commit_atomically_and_continue() {
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
            .all(|key| key != "variant_set" && !key.contains("enum_variant")),
        "no custom variant capability"
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
        json!({"frame_id":initial_frame,"target":"Selected.Count.Value","expression":"10"}),
    );
    assert_eq!(
        selected[0]["body"]["result"], "Choice.Count",
        "{selected:?}"
    );

    let current = frame(&mut server, &mut id);
    let empty = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"eMpTyVaLuE.cOuNt.vAlUe","expression":"4"}),
    );
    assert_eq!(empty[0]["body"]["result"], "Choice.Count", "{empty:?}");

    let current = frame(&mut server, &mut id);
    let nested = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"NestedValue.Count.Value","expression":"9"}),
    );
    assert_eq!(nested[0]["body"]["result"], "Choice.Count", "{nested:?}");

    let current = frame(&mut server, &mut id);
    let outcome = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Outcome.Error.value","expression":"'fail'"}),
    );
    assert_eq!(outcome[0]["body"]["result"], "Error(...)", "{outcome:?}");

    let current = frame(&mut server, &mut id);
    let failed = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Failed.Ok.value","expression":"9"}),
    );
    assert_eq!(failed[0]["body"]["result"], "Ok(...)", "{failed:?}");

    let current = frame(&mut server, &mut id);
    let optional = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Optional.Some.value","expression":"8"}),
    );
    assert_eq!(optional[0]["body"]["result"], "8", "{optional:?}");

    let current = frame(&mut server, &mut id);
    let missing = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Missing.Some.value","expression":"8"}),
    );
    assert_eq!(missing[0]["body"]["result"], "Some(...)", "{missing:?}");

    let current = frame(&mut server, &mut id);
    let packed = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Packed.Error.value","expression":"'pack'"}),
    );
    assert_eq!(packed[0]["body"]["result"], "Error(...)", "{packed:?}");

    let current = frame(&mut server, &mut id);
    let nested_result = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({
            "frame_id":current,
            "target":"NestedResult.Some.value.Error.value",
            "expression":"'inner'"
        }),
    );
    assert_eq!(
        nested_result[0]["body"]["result"], "Error(...)",
        "{nested_result:?}"
    );

    let current = frame(&mut server, &mut id);
    let holder = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({
            "frame_id":current,
            "target":"PackedHolder.Item.Count.Value",
            "expression":"3"
        }),
    );
    assert_eq!(holder[0]["body"]["result"], "Choice.Count", "{holder:?}");

    let current = frame(&mut server, &mut id);
    let items = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Items[0].Count.Value","expression":"0"}),
    );
    assert_eq!(items[0]["body"]["result"], "Choice.Count", "{items:?}");

    let current = frame(&mut server, &mut id);
    let scores = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({
            "frame_id":current,
            "target":"Scores['blue'].Count.Value",
            "expression":"16"
        }),
    );
    assert_eq!(scores[0]["body"]["result"], "Choice.Count", "{scores:?}");

    let global = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"target":"GlobalChoice.Count.Value","expression":"15"}),
    );
    assert_eq!(global[0]["body"]["result"], "Choice.Count", "{global:?}");

    let current = frame(&mut server, &mut id);
    let locals = locals_reference(&mut server, &mut id, current);
    let refreshed = send(
        &mut server,
        &mut id,
        "variables",
        json!({"variables_reference":locals,"start":0,"count":20}),
    );
    let selected_value = refreshed[0]["body"]["variables"]
        .as_array()
        .expect("locals")
        .iter()
        .find(|item| item["name"] == "Selected")
        .expect("Selected");
    assert_eq!(selected_value["value"], "Choice.Count");

    let current = frame(&mut server, &mut id);
    for (target, expression, code) in [
        ("PairValue.Value", "1", "variable_path_unsupported"),
        ("PairValue.Pair.Left", "1", "variable_path_unsupported"),
        ("PairValue.Empty", "1", "variable_path_unsupported"),
        ("Selected.Missing.Value", "1", "variable_target_unknown"),
        ("EmptyValue.Count.Nope", "1", "variable_target_unknown"),
        ("PairValue.Count.Value", "'wrong'", "variable_value_type"),
        ("Fixed.Count.Value", "1", "variable_not_mutable"),
    ] {
        let failed = send(
            &mut server,
            &mut id,
            "expression.set",
            json!({"frame_id":current,"target":target,"expression":expression}),
        );
        assert_eq!(
            failed[0]["error"]["code"], code,
            "{target} {expression}: {failed:?}"
        );
        assert!(
            failed[0]["error"]["help"]
                .as_str()
                .is_some_and(|hint| !hint.is_empty()),
            "{target} missing hint: {failed:?}"
        );
        let preserved = send(
            &mut server,
            &mut id,
            "evaluate",
            json!({"frame_id":current,"expression":"Selected"}),
        );
        assert_eq!(preserved[0]["success"], true, "{preserved:?}");
        assert_eq!(preserved[0]["body"]["result"], "Choice.Count");
    }

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let output = server
        .wait()
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(
        output,
        "10\n5\n4\n9\nfail\n9\n8\n8\npack\ninner\n3\n0\n16\n15\n99\n"
    );
}

#[test]
fn jsonl_variant_transition_stays_bound_to_the_selected_child_task() {
    const TASK_SOURCE: &str = r#"program TaskVariantTransition;

uses Std.Console, Std.Task;

function Work(): integer;
begin
  mutable var Optional: Option of integer := None;
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
    let executable = fpas_compiler::compile(&program).expect("compile task transition fixture");
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
        json!({"frame_id":child_frame,"target":"Optional.Some.value","expression":"4"}),
    ));
    assert_eq!(updated[0]["body"]["result"], "Some(...)", "{updated:?}");
    let expired_main = server.handle_line(&request(7, "scopes", json!({"frame_id":main_frame})));
    assert_eq!(expired_main[0]["error"]["code"], "unknown_frame");

    let _ = server.handle_line(&request(8, "continue", json!({})));
    let terminated = server.wait();
    assert!(
        terminated
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "4\n" })
    );
}

fn session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile transition session fixture");
    DebugSession::new(executable).expect("debug session")
}

fn session_scope(session: &mut DebugSession, name: &str) -> Option<u64> {
    let frame = session.stack(0, 1).ok()?.items.first()?.id;
    session
        .scopes(frame)
        .ok()?
        .into_iter()
        .find(|scope| scope.name == name)
        .map(|scope| scope.variables_reference)
}

fn step(session: &mut DebugSession) {
    assert!(matches!(
        session.step_into().expect("step"),
        DebugRunResult::Stopped(_)
    ));
}

fn qualified(root: &str, fields: &[&str]) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: root.to_string(),
        selectors: fields
            .iter()
            .map(|name| fpas_vm::DebugAssignmentSelector::Field((*name).to_string()))
            .collect(),
    }
}

#[test]
fn variant_transition_supports_mutable_parameters_and_capture_cells() {
    let mut parameter = session(
        r#"
program TransitionParameter;

type
  Choice = enum
    Empty;
    Count(Value: integer);
  end;

function ReadChoice(mutable Item: Choice): integer;
begin
  var Marker: integer := 0;
  case Item of
    Choice.Empty:
    begin
      return 0
    end;
    Choice.Count(Value):
    begin
      return Value
    end
  end
end;

begin
  var OutputValue: integer := ReadChoice(Choice.Empty);
  var Marker: integer := OutputValue
end.
"#,
    );
    let parameter_frame = loop {
        if session_scope(&mut parameter, "Parameters").is_some() {
            break parameter.stack(0, 1).expect("parameter stack").items[0].id;
        }
        step(&mut parameter);
    };
    parameter
        .set_expression(
            &qualified("Item", &["Count", "Value"]),
            &DebugExpression::Integer(5),
            Some(parameter_frame),
        )
        .expect("transition mutable enum parameter");
    assert!(matches!(
        parameter
            .step_out()
            .expect("return from parameter function"),
        DebugRunResult::Stopped(_)
    ));
    let locals = session_scope(&mut parameter, "Locals").expect("caller locals");
    let values = parameter.variables(locals, 0, 10).expect("caller values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "OutputValue")
            .expect("parameter result")
            .value,
        "5"
    );

    let mut capture = session(
        r#"
program TransitionCapture;

type
  Choice = enum
    Empty;
    Count(Value: integer);
  end;

function NextChoice(): function(): integer;
begin
  mutable var Selected: Choice := Choice.Empty;
  return function(): integer begin
    case Selected of
      Choice.Empty:
      begin
        return 0
      end;
      Choice.Count(Value):
      begin
        return Value
      end
    end
  end
end;

begin
  var Next: function(): integer := NextChoice();
  var First: integer := Next();
  var Marker: integer := First
end.
"#,
    );
    let (frame, _captures) = loop {
        if let Some(captures) = session_scope(&mut capture, "Captures") {
            break (
                capture.stack(0, 1).expect("capture stack").items[0].id,
                captures,
            );
        }
        step(&mut capture);
    };
    capture
        .set_expression(
            &qualified("Selected", &["Count", "Value"]),
            &DebugExpression::Integer(41),
            Some(frame),
        )
        .expect("transition captured enum");
    assert!(matches!(
        capture.step_out().expect("return from closure"),
        DebugRunResult::Stopped(_)
    ));
    let locals = session_scope(&mut capture, "Locals").expect("caller locals");
    let values = capture.variables(locals, 0, 10).expect("caller values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "First")
            .expect("closure result")
            .value,
        "41"
    );
}

#[test]
fn explicit_variant_name_wins_over_an_active_payload_field_collision() {
    let mut session = session(
        r#"
program TransitionCollision;

type
  Payload = record
    Value: integer;
  end;
  Choice = enum
    Holder(Count: Payload);
    Count(Value: integer);
  end;

begin
  var Initial: Payload := record
    Value := 1;
  end;
  mutable var Selected: Choice := Choice.Holder(Initial);
  var Marker: integer := 0
end.
"#,
    );
    let frame = loop {
        if let Some(locals) = session_scope(&mut session, "Locals") {
            let values = session.variables(locals, 0, 10).expect("collision locals");
            if values
                .items
                .iter()
                .any(|value| value.name == "Selected" && value.value == "Choice.Holder")
            {
                break session.stack(0, 1).expect("collision stack").items[0].id;
            }
        }
        step(&mut session);
    };

    session
        .set_expression(
            &qualified("Selected", &["Count", "Value"]),
            &DebugExpression::Integer(9),
            Some(frame),
        )
        .expect("explicit colliding variant transition");

    let locals = session_scope(&mut session, "Locals").expect("refreshed collision locals");
    let values = session.variables(locals, 0, 10).expect("collision values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "Selected")
            .expect("Selected after collision transition")
            .value,
        "Choice.Count"
    );
}
