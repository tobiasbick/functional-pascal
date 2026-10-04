//! Mutable local copies, nested value storage and captured cells.

use super::*;

#[test]
fn record_and_dictionary_descendants_rebuild_the_mutable_root() {
    let mut session = session(
        r#"program AggregateMutation;

type Box = record
  Value: integer;
  Other: integer;
end record;

type Container = record
  Items: array of (Box);
end record;

begin
   var Item: Box := Box(Value := 1, Other := 2);
   var Nested: Container := Container(Items := [Box(Value := 3, Other := 4)]);
   var Scores: dict of (string, integer) := ['Ada': 2, 'Grace': 5];
  const Marker: integer := 0;
end program;
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
fn parameter_copy_commit_is_observed_by_the_running_function() {
    let mut session = session(
        r#"
program ParameterMutation;

function ReadBack(InitialValue: integer): integer;
begin
   var Value: integer := InitialValue;
  return Value;
end function;

begin
  const OutputValue: integer := ReadBack(1);
  const Marker: integer := OutputValue;
end program;
"#,
    );
    while session_scope(&mut session, "Parameters").is_none() {
        step(&mut session);
    }
    step(&mut session);
    let frame = session.stack(0, 1).expect("parameter copy stack").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Value".to_string(),
                selectors: Vec::new(),
            },
            &DebugExpression::Integer(77),
            Some(frame),
        )
        .expect("textual mutable parameter-copy mutation");
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
   var Value: integer := 0;
  return function(): integer begin
    Value := Value + 1;
    return Value;
  end function;
end function;

begin
  const Next: function(): integer := Counter();
  const First: integer := Next();
  const Marker: integer := First;
end program;
"#,
    );
    let (frame, captures) = loop {
        if let Some(captures) = session_scope(&mut session, "Captures") {
            break (
                session.stack(0, 1).expect("capture stack").items[0].id,
                captures,
            );
        }
        step(&mut session);
    };
    assert_eq!(
        session.variables(captures, 0, 10).expect("captures").items[0].value,
        "0"
    );
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Value".to_string(),
                selectors: Vec::new(),
            },
            &DebugExpression::Integer(40),
            Some(frame),
        )
        .expect("textual capture mutation");

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

#[test]
fn dictionary_structure_mutation_supports_parameter_copies_and_capture_cells() {
    let mut parameter_session = session(
        r#"
program DictionaryParameterMutation;

function ReadAdded(InitialScores: dict of (string, integer)): integer;
begin
   var Scores: dict of (string, integer) := InitialScores;
  const Marker: integer := Scores['Seed'];
  return Scores['Added'] + Marker;
end function;

begin
  const OutputValue: integer := ReadAdded(['Seed': 1]);
  const Marker: integer := OutputValue;
end program;
"#,
    );
    while session_scope(&mut parameter_session, "Parameters").is_none() {
        step(&mut parameter_session);
    }
    step(&mut parameter_session);
    let parameter_frame = parameter_session
        .stack(0, 1)
        .expect("parameter copy dictionary stack")
        .items[0]
        .id;
    parameter_session
        .insert_dictionary_entry(
            &DebugAssignmentTarget {
                root: "Scores".to_string(),
                selectors: Vec::new(),
            },
            &DebugExpression::String("Added".to_string()),
            &DebugExpression::Integer(8),
            Some(parameter_frame),
        )
        .expect("insert into mutable dictionary parameter copy");
    assert!(matches!(
        parameter_session
            .step_out()
            .expect("return from parameter function"),
        DebugRunResult::Stopped(_)
    ));
    let locals = session_scope(&mut parameter_session, "Locals").expect("caller locals");
    let values = parameter_session
        .variables(locals, 0, 10)
        .expect("caller values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "OutputValue")
            .expect("parameter function result")
            .value,
        "9"
    );

    let mut capture_session = session(
        r#"
program DictionaryCaptureMutation;

function Reader(): function(): integer;
begin
   var Scores: dict of (string, integer) := ['Seed': 1];
  return function(): integer begin
    const Marker: integer := Scores['Seed'];
    return Scores['Added'] + Marker;
  end function;
end function;

begin
  const ReadValue: function(): integer := Reader();
  const OutputValue: integer := ReadValue();
  const Marker: integer := OutputValue;
end program;
"#,
    );
    let capture_frame = loop {
        if session_scope(&mut capture_session, "Captures").is_some() {
            break capture_session
                .stack(0, 1)
                .expect("capture dictionary stack")
                .items[0]
                .id;
        }
        step(&mut capture_session);
    };
    capture_session
        .insert_dictionary_entry(
            &DebugAssignmentTarget {
                root: "Scores".to_string(),
                selectors: Vec::new(),
            },
            &DebugExpression::String("Added".to_string()),
            &DebugExpression::Integer(8),
            Some(capture_frame),
        )
        .expect("insert into captured dictionary");
    assert!(matches!(
        capture_session
            .step_out()
            .expect("return from dictionary closure"),
        DebugRunResult::Stopped(_)
    ));
    let locals = session_scope(&mut capture_session, "Locals").expect("capture caller locals");
    let values = capture_session
        .variables(locals, 0, 10)
        .expect("capture caller values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "OutputValue")
            .expect("capture function result")
            .value,
        "9"
    );
}
