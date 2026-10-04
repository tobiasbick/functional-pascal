//! Shared evaluation limits and cancellation for variable mutation.

use super::*;

#[test]
fn dictionary_structure_mutation_obeys_shared_limits_effect_policy_and_cancellation() {
    let mut session = session(
        r#"
program DictionaryMutationLimits;

uses Std.Console as Console;

function Forever(): integer;
begin
  while true do begin null; end; end while;
  return 0;
end function;

procedure Emit();
begin
  Console.WriteLn('not live');
end procedure;

begin
   var Scores: dict of (string, integer) := ['Seed': 1];
  const Marker: integer := Scores['Seed'];
end program;
"#,
    );
    let frame = loop {
        if let Some(locals) = session_scope(&mut session, "Locals") {
            let ready = session
                .variables(locals, 0, 10)
                .expect("dictionary locals")
                .items
                .iter()
                .any(|value| value.name == "Scores" && value.value == "{1 entries}");
            if ready {
                break session.stack(0, 1).expect("dictionary stack").items[0].id;
            }
        }
        step(&mut session);
    };
    let target = DebugAssignmentTarget {
        root: "Scores".to_string(),
        selectors: Vec::new(),
    };

    let limited = DebugEvaluationLimits {
        max_operations: 1,
        ..DebugEvaluationLimits::default()
    };
    assert_eq!(
        session
            .insert_dictionary_entry_with_limits(
                &target,
                &DebugExpression::String("Limited".to_string()),
                &DebugExpression::Integer(2),
                Some(frame),
                limited,
            )
            .expect_err("shared key and value operation limit")
            .kind,
        DebugErrorKind::EvaluationLimit
    );

    let forbidden = DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable("Emit".to_string())),
        arguments: Vec::new(),
    };
    assert_eq!(
        session
            .insert_dictionary_entry(
                &target,
                &DebugExpression::String("Effect".to_string()),
                &forbidden,
                Some(frame),
            )
            .expect_err("effectful dictionary value")
            .kind,
        DebugErrorKind::ForbiddenCallEffect
    );
    assert!(session.output().lines.is_empty(), "sandbox emitted output");

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
        .insert_dictionary_entry_with_limits(
            &target,
            &DebugExpression::String("Cancelled".to_string()),
            &forever,
            Some(frame),
            DebugEvaluationLimits {
                call_timeout: Duration::from_secs(1),
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("cancelled dictionary value");
    cancellation.join().expect("cancellation thread");
    assert_eq!(cancelled.kind, DebugErrorKind::CallCancelled);
    assert_eq!(
        session
            .evaluate(&DebugExpression::Name("Scores".to_string()), Some(frame))
            .expect("dictionary after failed mutations")
            .value,
        "{1 entries}"
    );
    assert!(session.scopes(frame).is_ok(), "failures preserve frame");
}

#[test]
fn textual_selectors_share_one_call_and_limit_budget_before_commit() {
    let mut session = session(
        r#"
program SelectorMutation;

uses Std.Console as Console;

function ChooseIndex(): integer;
begin
  return 1;
end function;

procedure Emit();
begin
  Console.WriteLn('not live');
end procedure;

begin
   var Items: array of (integer) := [1, 2];
  const Marker: integer := Items[0];
end program;
"#,
    );
    let frame = loop {
        if let Some(locals) = session_scope(&mut session, "Locals") {
            let ready = session
                .variables(locals, 0, 10)
                .expect("selector locals")
                .items
                .iter()
                .any(|value| value.name == "Items" && value.value != "<uninitialized>");
            if ready {
                break session.stack(0, 1).expect("selector stack").items[0].id;
            }
        }
        step(&mut session);
    };
    let called_target = DebugAssignmentTarget {
        root: "Items".to_string(),
        selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Call {
            callee: Box::new(DebugExpression::Callable("ChooseIndex".to_string())),
            arguments: Vec::new(),
        })],
    };
    let once = DebugEvaluationLimits {
        max_calls: 1,
        ..DebugEvaluationLimits::default()
    };
    assert_eq!(
        session
            .set_expression_with_limits(
                &called_target,
                &DebugExpression::Integer(9),
                Some(frame),
                once,
            )
            .expect("one selector call")
            .value,
        "9"
    );

    let frame = session.stack(0, 1).expect("fresh selector stack").items[0].id;
    let first = DebugAssignmentTarget {
        root: "Items".to_string(),
        selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
    };
    let limited = DebugEvaluationLimits {
        max_operations: 1,
        ..DebugEvaluationLimits::default()
    };
    assert_eq!(
        session
            .set_expression_with_limits(&first, &DebugExpression::Integer(8), Some(frame), limited,)
            .expect_err("combined selector/replacement budget")
            .kind,
        DebugErrorKind::EvaluationLimit
    );
    assert!(
        session.scopes(frame).is_ok(),
        "limit failure preserves frame"
    );

    let forbidden = DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable("Emit".to_string())),
        arguments: Vec::new(),
    };
    assert_eq!(
        session
            .set_expression(&first, &forbidden, Some(frame))
            .expect_err("effectful replacement")
            .kind,
        DebugErrorKind::ForbiddenCallEffect
    );
    assert!(
        session.scopes(frame).is_ok(),
        "effect failure preserves frame"
    );
    assert!(
        session.output().lines.is_empty(),
        "sandbox call produced no output"
    );
}
