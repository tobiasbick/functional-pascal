//! Storage-root variant transitions and payload-name resolution.

use fpas_vm::{DebugAssignmentTarget, DebugExpression, DebugRunResult, DebugSession};

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
fn variant_transition_supports_parameter_copies_and_capture_cells() {
    let mut parameter = session(
        r#"program TransitionParameter;

type Choice = enum
  Empty;
  Count(Value: integer);
end enum;

function ReadChoice(InitialItem: Choice): integer;
begin
   var Item: Choice := InitialItem;
  const Marker: integer := 0;
  case Item of
    when Choice.Empty:
      begin
        return 0;
      end;
    when Choice.Count(const Value):
      begin
        return Value;
      end;
  end case;
end function;

begin
  const OutputValue: integer := ReadChoice(Choice.Empty);
  const Marker: integer := OutputValue;
end program;
"#,
    );
    while session_scope(&mut parameter, "Parameters").is_none() {
        step(&mut parameter);
    }
    step(&mut parameter);
    let parameter_frame = parameter.stack(0, 1).expect("parameter copy stack").items[0].id;
    parameter
        .set_expression(
            &qualified("Item", &["Count", "Value"]),
            &DebugExpression::Integer(5),
            Some(parameter_frame),
        )
        .expect("transition mutable enum parameter copy");
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
        r#"program TransitionCapture;

type Choice = enum
  Empty;
  Count(Value: integer);
end enum;

function NextChoice(): function(): integer;
begin
   var Selected: Choice := Choice.Empty;
  return function(): integer begin
    case Selected of
      when Choice.Empty:
        begin
          return 0;
        end;
      when Choice.Count(const Value):
        begin
          return Value;
        end;
    end case;
  end function;
end function;

begin
  const Next: function(): integer := NextChoice();
  const First: integer := Next();
  const Marker: integer := First;
end program;
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
        r#"program TransitionCollision;

type Payload = record
  Value: integer;
end record;

type Choice = enum
  Holder(Count: Payload);
  Count(Value: integer);
end enum;

begin
  const Initial: Payload := Payload(Value := 1);
   var Selected: Choice := Choice.Holder(Initial);
  const Marker: integer := 0;
end program;
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
