//! JSONL variable-mutation capability, success, failure, and generation contracts.

#![allow(
    clippy::expect_used,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, jsonl::JsonlServer};
use fpas_vm::{DebugExpression, DebugRunResult, DebugSession};
use serde_json::{Value, json};

fn server() -> JsonlServer {
    let source = "program Main;\n\nfunction Twice(Value: integer): integer;\nbegin\n  return Value * 2\nend;\n\nbegin\n  mutable var X: integer := 1;\n  var Fixed: integer := 2;\n  X := X + Fixed\nend.";
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile mutation fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile mutation fixture");
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

fn locals_reference(server: &mut JsonlServer, id: &mut u64) -> u64 {
    *id += 1;
    let stack = server.handle_line(&request(*id, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("frame");
    *id += 1;
    let scopes = server.handle_line(&request(*id, "scopes", json!({"frame_id":frame})));
    scopes[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find(|scope| scope["name"] == "Locals")
        .and_then(|scope| scope["variables_reference"].as_u64())
        .expect("locals")
}

#[test]
fn jsonl_sets_variables_with_stable_errors_and_fresh_handles() {
    let mut server = server();
    let initialized = server.handle_line(&request(1, "initialize", json!({"version":2})));
    assert_eq!(initialized[0]["body"]["capabilities"]["set_variable"], true);
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));
    let _ = server.handle_line(&request(3, "step_into", json!({})));
    let _ = server.wait();

    let mut id = 3;
    let locals = locals_reference(&mut server, &mut id);
    let variables = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variables",
        json!({"variables_reference":locals}),
    ));
    assert_eq!(variables[0]["body"]["variables"][0]["value"], "1");

    let missing = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"X"}),
    ));
    assert_eq!(missing[0]["error"]["code"], "invalid_request");

    let unknown = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"Missing","expression":"1"}),
    ));
    assert_eq!(unknown[0]["error"]["code"], "variable_target_unknown");

    let wrong = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"X","expression":"'wrong'"}),
    ));
    assert_eq!(wrong[0]["error"]["code"], "variable_value_type");

    let updated = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"X","expression":"Twice(21)"}),
    ));
    assert_eq!(updated[0]["body"]["result"], "42", "{updated:?}");

    let expired = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"X","expression":"1"}),
    ));
    assert_eq!(expired[0]["error"]["code"], "variable_target_expired");

    let locals = locals_reference(&mut server, &mut id);
    let fixed = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variable.set",
        json!({"variables_reference":locals,"name":"Fixed","expression":"3"}),
    ));
    assert_eq!(fixed[0]["error"]["code"], "variable_uninitialized");

    let variables = server.handle_line(&request(
        {
            id += 1;
            id
        },
        "variables",
        json!({"variables_reference":locals}),
    ));
    assert_eq!(variables[0]["body"]["variables"][0]["value"], "42");
}

#[test]
fn record_and_dictionary_descendants_rebuild_the_mutable_root() {
    let mut session = session(
        r#"
program AggregateMutation;

type
  Box = record
    Value: integer;
    Other: integer;
  end;
  Container = record
    Items: array of Box;
  end;

begin
  mutable var Item: Box := record
    Value := 1;
    Other := 2;
  end;
  mutable var Nested: Container := record
    Items := [record
      Value := 3;
      Other := 4;
    end];
  end;
  mutable var Scores: dict of string to integer := ['Ada': 2, 'Grace': 5];
  var Marker: integer := 0
end.
"#,
    );
    let locals = loop {
        if let Some(locals) = session_scope(&mut session, "Locals") {
            let values = session.variables(locals, 0, 10).expect("locals");
            let ready = values
                .items
                .iter()
                .any(|value| value.name == "Scores" && value.value != "<uninitialized>");
            if ready {
                break locals;
            }
        }
        step(&mut session);
    };
    let values = session.variables(locals, 0, 10).expect("aggregate locals");
    let item = values
        .items
        .iter()
        .find(|value| value.name == "Item")
        .expect("record")
        .variables_reference;
    session
        .set_variable(item, "Value", &DebugExpression::Integer(7))
        .expect("record field mutation");
    let locals = session_scope(&mut session, "Locals").expect("fresh locals");
    let values = session.variables(locals, 0, 10).expect("fresh values");
    let scores = values
        .items
        .iter()
        .find(|value| value.name == "Scores")
        .expect("fresh dictionary")
        .variables_reference;
    let item = values
        .items
        .iter()
        .find(|value| value.name == "Item")
        .expect("fresh record")
        .variables_reference;
    let fields = session.variables(item, 0, 10).expect("record fields");
    assert_eq!(fields.items[0].value, "7");
    assert_eq!(
        fields.items[1].value, "2",
        "other record field is preserved"
    );

    session
        .set_variable(scores, "[0].value", &DebugExpression::Integer(9))
        .expect("dictionary value mutation");
    let locals = session_scope(&mut session, "Locals").expect("fresh locals");
    let scores = session
        .variables(locals, 0, 10)
        .expect("fresh values")
        .items
        .into_iter()
        .find(|value| value.name == "Scores")
        .expect("fresh dictionary")
        .variables_reference;
    let entries = session
        .variables(scores, 0, 10)
        .expect("dictionary entries");
    assert_eq!(entries.items[1].value, "9");
    assert_eq!(
        entries.items[3].value, "5",
        "other dictionary entry is preserved"
    );
    assert_eq!(
        session
            .set_variable(
                scores,
                "[0].key",
                &DebugExpression::String("Grace".to_string()),
            )
            .expect_err("dictionary key is synthetic")
            .kind,
        fpas_vm::DebugErrorKind::VariablePathUnsupported
    );

    let locals = session_scope(&mut session, "Locals").expect("fresh locals");
    let nested = session
        .variables(locals, 0, 10)
        .expect("fresh values")
        .items
        .into_iter()
        .find(|value| value.name == "Nested")
        .expect("nested record")
        .variables_reference;
    let nested_items = session
        .variables(nested, 0, 10)
        .expect("container fields")
        .items[0]
        .variables_reference;
    let nested_box = session
        .variables(nested_items, 0, 10)
        .expect("nested array")
        .items[0]
        .variables_reference;
    session
        .set_variable(nested_box, "Value", &DebugExpression::Integer(11))
        .expect("nested aggregate mutation");
    let locals = session_scope(&mut session, "Locals").expect("fresh locals");
    let nested = session
        .variables(locals, 0, 10)
        .expect("fresh values")
        .items
        .into_iter()
        .find(|value| value.name == "Nested")
        .expect("nested record")
        .variables_reference;
    let nested_items = session
        .variables(nested, 0, 10)
        .expect("container fields")
        .items[0]
        .variables_reference;
    let nested_box = session
        .variables(nested_items, 0, 10)
        .expect("nested array")
        .items[0]
        .variables_reference;
    let nested_fields = session
        .variables(nested_box, 0, 10)
        .expect("nested record fields");
    assert_eq!(nested_fields.items[0].value, "11");
    assert_eq!(nested_fields.items[1].value, "4");
}

#[test]
fn mutable_parameter_commit_is_observed_by_the_running_function() {
    let mut session = session(
        r#"
program ParameterMutation;

function ReadBack(mutable Value: integer): integer;
begin
  return Value
end;

begin
  var OutputValue: integer := ReadBack(1);
  var Marker: integer := OutputValue
end.
"#,
    );
    let parameters = loop {
        if let Some(parameters) = session_scope(&mut session, "Parameters") {
            break parameters;
        }
        step(&mut session);
    };
    session
        .set_variable(parameters, "Value", &DebugExpression::Integer(77))
        .expect("mutable parameter mutation");
    assert!(matches!(
        session.step_out().expect("return to caller"),
        DebugRunResult::Stopped(_)
    ));
    let locals = session_scope(&mut session, "Locals").expect("caller locals");
    let values = session.variables(locals, 0, 10).expect("caller values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "OutputValue")
            .expect("function result")
            .value,
        "77"
    );
}

#[test]
fn mutable_capture_commit_preserves_the_existing_cell_alias() {
    let mut session = session(
        r#"
program CaptureMutation;

function Counter(): function(): integer;
begin
  mutable var Value: integer := 0;
  return function(): integer begin
    Value := Value + 1;
    return Value
  end
end;

begin
  var Next: function(): integer := Counter();
  var First: integer := Next();
  var Marker: integer := First
end.
"#,
    );
    let captures = loop {
        if let Some(captures) = session_scope(&mut session, "Captures") {
            break captures;
        }
        step(&mut session);
    };
    assert_eq!(
        session.variables(captures, 0, 10).expect("captures").items[0].value,
        "0"
    );
    session
        .set_variable(captures, "Value", &DebugExpression::Integer(40))
        .expect("capture mutation");

    assert!(matches!(
        session.step_out().expect("return from closure"),
        DebugRunResult::Stopped(_)
    ));
    let locals = session_scope(&mut session, "Locals").expect("caller locals");
    let values = session.variables(locals, 0, 10).expect("caller values");
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
