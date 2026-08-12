//! Protocol-neutral sequence mutation coverage for storage roots and shared limits.

#![allow(
    clippy::expect_used,
    reason = "session tests keep fixture failures local"
)]

use std::{thread, time::Duration};

use fpas_vm::{
    DebugAssignmentSelector, DebugAssignmentTarget, DebugErrorKind, DebugEvaluationLimits,
    DebugExpression, DebugRunResult, DebugSession,
};

fn session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile sequence session fixture");
    DebugSession::new(executable).expect("debug session")
}

fn scope(session: &mut DebugSession, name: &str) -> Option<u64> {
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

fn frame_with_scope(session: &mut DebugSession, name: &str) -> u64 {
    loop {
        if scope(session, name).is_some() {
            return session.stack(0, 1).expect("stack").items[0].id;
        }
        step(session);
    }
}

fn root(name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: name.to_string(),
        selectors: Vec::new(),
    }
}

#[test]
fn sequence_mutation_supports_parameters_and_capture_cells() {
    let mut parameter = session(
        r#"
program ArrayParameterMutation;

function ReadAdded(mutable Values: array of integer): integer;
begin
  var Marker: integer := Values[0];
  return Values[1] + Marker
end;

begin
  var OutputValue: integer := ReadAdded([1]);
  var Marker: integer := OutputValue
end.
"#,
    );
    let parameter_frame = frame_with_scope(&mut parameter, "Parameters");
    parameter
        .insert_array_element(
            &root("Values"),
            &DebugExpression::Integer(1),
            &DebugExpression::Integer(8),
            Some(parameter_frame),
        )
        .expect("insert into mutable array parameter");
    assert!(matches!(
        parameter
            .step_out()
            .expect("return from parameter function"),
        DebugRunResult::Stopped(_)
    ));
    let locals = scope(&mut parameter, "Locals").expect("caller locals");
    let values = parameter.variables(locals, 0, 10).expect("caller values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "OutputValue")
            .expect("parameter result")
            .value,
        "9"
    );

    let mut capture = session(
        r#"
program StringCaptureMutation;

function Reader(): function(): string;
begin
  mutable var Text: string := 'A😀B';
  return function(): string begin
    var Marker: string := Text;
    return Text
  end
end;

begin
  var ReadValue: function(): string := Reader();
  var OutputValue: string := ReadValue();
  var Marker: string := OutputValue
end.
"#,
    );
    let capture_frame = frame_with_scope(&mut capture, "Captures");
    capture
        .replace_string_character(
            &root("Text"),
            &DebugExpression::Integer(1),
            &DebugExpression::String("é".to_string()),
            Some(capture_frame),
        )
        .expect("replace captured string character");
    assert!(matches!(
        capture.step_out().expect("return from capture"),
        DebugRunResult::Stopped(_)
    ));
    let locals = scope(&mut capture, "Locals").expect("capture caller locals");
    let values = capture.variables(locals, 0, 10).expect("capture values");
    assert_eq!(
        values
            .items
            .iter()
            .find(|value| value.name == "OutputValue")
            .expect("capture result")
            .value,
        "'AéB'"
    );
}

#[test]
fn sequence_mutation_supports_global_and_nested_stored_targets() {
    let mut session = session(
        r#"
program NestedSequenceMutation;

type
  Container = record
    Items: array of integer;
  end;

mutable var
  GlobalValues: array of integer := [4, 6];

begin
  mutable var Nested: Container := record
    Items := [1, 3];
  end;
  var Marker: integer := Nested.Items[0] + GlobalValues[0]
end.
"#,
    );
    while session
        .evaluate(&DebugExpression::Name("GlobalValues".to_string()), None)
        .is_err()
    {
        step(&mut session);
    }
    session
        .insert_array_element(
            &root("GlobalValues"),
            &DebugExpression::Integer(1),
            &DebugExpression::Integer(5),
            None,
        )
        .expect("insert into global array");
    let frame = loop {
        if let Some(locals) = scope(&mut session, "Locals") {
            let ready = session
                .variables(locals, 0, 10)
                .expect("locals")
                .items
                .iter()
                .any(|value| value.name == "Nested" && value.value != "<uninitialized>");
            if ready {
                break session.stack(0, 1).expect("stack").items[0].id;
            }
        }
        step(&mut session);
    };
    let target = DebugAssignmentTarget {
        root: "Nested".to_string(),
        selectors: vec![DebugAssignmentSelector::Field("Items".to_string())],
    };
    let inserted = session
        .insert_array_element(
            &target,
            &DebugExpression::Integer(1),
            &DebugExpression::Integer(2),
            Some(frame),
        )
        .expect("insert into nested global array");
    assert_eq!(inserted.array.value, "[3 items]");
    let current = session.stack(0, 1).expect("fresh stack").items[0].id;
    assert_eq!(
        session
            .evaluate(
                &DebugExpression::Index {
                    base: Box::new(DebugExpression::Field {
                        base: Box::new(DebugExpression::Name("Nested".to_string())),
                        name: "Items".to_string(),
                    }),
                    index: Box::new(DebugExpression::Integer(1)),
                },
                Some(current),
            )
            .expect("nested inserted value")
            .value,
        "2"
    );
}

#[test]
fn sequence_mutation_obeys_shared_limits_effect_policy_and_cancellation() {
    let mut session = session(
        r#"
program SequenceMutationLimits;

uses Std.Console;

function Forever(): integer;
begin
  while true do begin end;
  return 0
end;

procedure Emit();
begin
  WriteLn('not live')
end;

begin
  mutable var Values: array of integer := [1];
  var Marker: integer := Values[0]
end.
"#,
    );
    let frame = loop {
        if let Some(locals) = scope(&mut session, "Locals") {
            let ready = session
                .variables(locals, 0, 10)
                .expect("locals")
                .items
                .iter()
                .any(|value| value.name == "Values" && value.value == "[1 items]");
            if ready {
                break session.stack(0, 1).expect("stack").items[0].id;
            }
        }
        step(&mut session);
    };
    let limited = DebugEvaluationLimits {
        max_operations: 1,
        ..DebugEvaluationLimits::default()
    };
    assert_eq!(
        session
            .insert_array_element_with_limits(
                &root("Values"),
                &DebugExpression::Integer(0),
                &DebugExpression::Integer(2),
                Some(frame),
                limited,
            )
            .expect_err("shared operation limit")
            .kind,
        DebugErrorKind::EvaluationLimit
    );
    let forbidden = DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable("Emit".to_string())),
        arguments: Vec::new(),
    };
    assert_eq!(
        session
            .insert_array_element(
                &root("Values"),
                &DebugExpression::Integer(0),
                &forbidden,
                Some(frame),
            )
            .expect_err("forbidden effect")
            .kind,
        DebugErrorKind::ForbiddenCallEffect
    );
    assert!(session.output().lines.is_empty());

    let handle = session.evaluation_cancel_handle();
    let cancellation = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        handle.cancel();
    });
    let forever = DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable("Forever".to_string())),
        arguments: Vec::new(),
    };
    let cancelled = session
        .insert_array_element_with_limits(
            &root("Values"),
            &DebugExpression::Integer(0),
            &forever,
            Some(frame),
            DebugEvaluationLimits {
                call_timeout: Duration::from_secs(1),
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("cancelled element expression");
    cancellation.join().expect("cancellation thread");
    assert_eq!(cancelled.kind, DebugErrorKind::CallCancelled);
    assert_eq!(
        session
            .evaluate(&DebugExpression::Name("Values".to_string()), Some(frame))
            .expect("array after failures")
            .value,
        "[1 items]"
    );
    assert!(session.scopes(frame).is_ok());
}
