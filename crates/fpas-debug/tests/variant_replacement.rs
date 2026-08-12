//! JSONL complete enum, Result, and Option replacement coverage.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use fpas_vm::{DebugAssignmentTarget, DebugExpression, DebugRunResult, DebugSession};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../tests/debugger/fixtures/variant_replacement.fpas");

fn server() -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile variant fixture");
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
    panic!("variant fixture locals never became initialized")
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
fn jsonl_variant_replacements_commit_atomically_and_continue() {
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
        json!({"frame_id":initial_frame,"target":"Selected","expression":"Choice.Pair(10, 20)"}),
    );
    assert_eq!(selected[0]["body"]["result"], "Choice.Pair", "{selected:?}");
    assert_eq!(selected[0]["body"]["named_variables"], 2);

    let current = frame(&mut server, &mut id);
    let locals = locals_reference(&mut server, &mut id, current);
    let pair_value = send(
        &mut server,
        &mut id,
        "variable.set",
        json!({"variables_reference":locals,"name":"PairValue","expression":"Choice.Empty"}),
    );
    assert_eq!(
        pair_value[0]["body"]["result"], "Choice.Empty",
        "{pair_value:?}"
    );

    let current = frame(&mut server, &mut id);
    let empty = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"eMpTyVaLuE","expression":"cHoIcE.cOuNt(4)"}),
    );
    assert_eq!(empty[0]["body"]["result"], "Choice.Count", "{empty:?}");

    let current = frame(&mut server, &mut id);
    let outcome = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Outcome","expression":"Error('fail')"}),
    );
    assert_eq!(outcome[0]["body"]["result"], "Error(...)", "{outcome:?}");

    let current = frame(&mut server, &mut id);
    let failed = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Failed","expression":"Ok(9)"}),
    );
    assert_eq!(failed[0]["body"]["result"], "Ok(...)", "{failed:?}");

    let current = frame(&mut server, &mut id);
    let optional = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Optional","expression":"None"}),
    );
    assert_eq!(optional[0]["body"]["result"], "None", "{optional:?}");

    let current = frame(&mut server, &mut id);
    let missing = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Missing","expression":"Some(8)"}),
    );
    assert_eq!(missing[0]["body"]["result"], "Some(...)", "{missing:?}");

    let current = frame(&mut server, &mut id);
    let packed = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Packed","expression":"Error('pack')"}),
    );
    assert_eq!(packed[0]["body"]["result"], "Error(...)", "{packed:?}");

    let current = frame(&mut server, &mut id);
    let nested_result = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"NestedResult","expression":"None"}),
    );
    assert_eq!(
        nested_result[0]["body"]["result"], "None",
        "{nested_result:?}"
    );

    let current = frame(&mut server, &mut id);
    let holder = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"PackedHolder.Item","expression":"Choice.Pair(1, 2)"}),
    );
    assert_eq!(holder[0]["body"]["result"], "Choice.Pair", "{holder:?}");

    let current = frame(&mut server, &mut id);
    let items = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"frame_id":current,"target":"Items[0]","expression":"Choice.Empty"}),
    );
    assert_eq!(items[0]["body"]["result"], "Choice.Empty", "{items:?}");

    let current = frame(&mut server, &mut id);
    let scores = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({
            "frame_id":current,
            "target":"Scores['blue']",
            "expression":"Choice.Count(16)"
        }),
    );
    assert_eq!(scores[0]["body"]["result"], "Choice.Count", "{scores:?}");

    let global = send(
        &mut server,
        &mut id,
        "expression.set",
        json!({"target":"GlobalChoice","expression":"Choice.Pair(7, 8)"}),
    );
    assert_eq!(global[0]["body"]["result"], "Choice.Pair", "{global:?}");

    let current = frame(&mut server, &mut id);
    for (target, expression, code) in [
        ("Selected", "Pair(1, 2)", "call_target_unknown"),
        ("Selected", "Choice.Missing", "call_target_unknown"),
        ("Selected", "Choice.Pair(1)", "call_arity"),
        ("Selected", "Choice.Count('wrong')", "evaluation_type"),
        ("Selected", "1", "variable_value_type"),
        ("Fixed", "Choice.Empty", "variable_not_mutable"),
        ("Selected.Value", "1", "variable_target_unknown"),
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
        let preserved = send(
            &mut server,
            &mut id,
            "evaluate",
            json!({"frame_id":current,"expression":"Selected"}),
        );
        assert_eq!(preserved[0]["success"], true, "{preserved:?}");
        assert_eq!(preserved[0]["body"]["result"], "Choice.Pair");
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
        "30\n0\n4\n9\nfail\n9\n0\n8\npack\n0\n3\n0\n16\n15\n99\n"
    );
}

#[test]
fn jsonl_variant_replacement_stays_bound_to_the_selected_child_task() {
    const TASK_SOURCE: &str = r#"program TaskVariantReplacement;

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
    let executable = fpas_compiler::compile(&program).expect("compile task variant fixture");
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
        json!({"frame_id":child_frame,"target":"Optional","expression":"None"}),
    ));
    assert_eq!(updated[0]["body"]["result"], "None", "{updated:?}");
    let expired_main = server.handle_line(&request(7, "scopes", json!({"frame_id":main_frame})));
    assert_eq!(expired_main[0]["error"]["code"], "unknown_frame");

    let _ = server.handle_line(&request(8, "continue", json!({})));
    let terminated = server.wait();
    assert!(
        terminated
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "0\n" })
    );
}

fn session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile variant session fixture");
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

fn root(name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: name.to_string(),
        selectors: Vec::new(),
    }
}

#[test]
fn variant_replacement_supports_mutable_parameters_and_capture_cells() {
    let mut parameter = session(
        r#"
program VariantParameter;

type
  Choice = enum
    Count(Value: integer);
    Pair(Left: integer; Right: integer);
  end;

function ReadChoice(mutable Item: Choice): integer;
begin
  var Marker: integer := 0;
  case Item of
    Choice.Count(Value):
    begin
      return Value
    end;
    Choice.Pair(Left, Right):
    begin
      return Left + Right
    end
  end
end;

begin
  var OutputValue: integer := ReadChoice(Choice.Count(1));
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
            &root("Item"),
            &DebugExpression::Call {
                callee: Box::new(DebugExpression::Callable("Choice.Pair".to_string())),
                arguments: vec![DebugExpression::Integer(2), DebugExpression::Integer(3)],
            },
            Some(parameter_frame),
        )
        .expect("replace mutable enum parameter");
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
program VariantCapture;

type
  Choice = enum
    Count(Value: integer);
    Pair(Left: integer; Right: integer);
  end;

function NextChoice(): function(): integer;
begin
  mutable var Selected: Choice := Choice.Count(1);
  return function(): integer begin
    case Selected of
      Choice.Count(Value):
      begin
        return Value
      end;
      Choice.Pair(Left, Right):
      begin
        return Left + Right
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
            &root("Selected"),
            &DebugExpression::Call {
                callee: Box::new(DebugExpression::Callable("Choice.Pair".to_string())),
                arguments: vec![DebugExpression::Integer(20), DebugExpression::Integer(21)],
            },
            Some(frame),
        )
        .expect("replace captured enum");
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
