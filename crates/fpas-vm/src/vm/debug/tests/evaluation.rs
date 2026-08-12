use super::*;
use fpas_bytecode::Value;

#[test]
fn evaluation_resolves_shadowed_frame_values_and_globals() {
    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize locals"));
    let frame = session.stack(0, 1).expect("stack").items[0].id;

    let shadowed = session
        .evaluate(&DebugExpression::Name("answer".to_string()), Some(frame))
        .expect("shadowed local");
    assert_eq!(shadowed.value, "'boom'");

    let global = session
        .evaluate(&DebugExpression::Name("g".to_string()), None)
        .expect("global");
    assert_eq!(global.value, "42");
}

#[test]
fn evaluation_reads_aggregate_indexes_and_expands_results() {
    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize locals"));
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let expression = DebugExpression::Index {
        base: Box::new(DebugExpression::Name("Items".to_string())),
        index: Box::new(DebugExpression::Integer(1)),
    };

    let result = session
        .evaluate(&expression, Some(frame))
        .expect("array index");

    assert_eq!(result.value, "2");
}

#[test]
fn evaluation_uses_shared_numeric_value_semantics() {
    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize locals"));
    let expression = DebugExpression::Binary {
        operation: DebugBinaryOperation::Add,
        left: Box::new(DebugExpression::Name("G".to_string())),
        right: Box::new(DebugExpression::Integer(1)),
    };

    let result = session
        .evaluate(&expression, None)
        .expect("global arithmetic");

    assert_eq!(result.value, "43");
}

#[test]
fn evaluation_rejects_stale_frames_and_non_boolean_conditions() {
    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize locals"));
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    stopped(session.step_into().expect("enter helper"));

    let stale = session
        .evaluate(&DebugExpression::Integer(1), Some(frame))
        .expect_err("stale frame");
    assert_eq!(stale.kind, DebugErrorKind::UnknownFrame);

    let current = session.stack(0, 1).expect("callee stack").items[0].id;
    let wrong_type = session
        .evaluate_boolean(&DebugExpression::Integer(1), Some(current))
        .expect_err("strict Boolean condition");
    assert_eq!(wrong_type.kind, DebugErrorKind::EvaluationType);
}

#[test]
fn evaluation_enforces_operation_limits_without_mutating_execution() {
    let mut session = DebugSession::new(inspection_executable()).expect("debug session");
    stopped(session.step_into().expect("initialize locals"));
    let before = session.output().lines;
    let expression = DebugExpression::Binary {
        operation: DebugBinaryOperation::Add,
        left: Box::new(DebugExpression::Integer(1)),
        right: Box::new(DebugExpression::Integer(2)),
    };
    let limits = DebugEvaluationLimits {
        max_operations: 1,
        ..DebugEvaluationLimits::default()
    };

    let error = session
        .evaluate_with_limits(&expression, None, limits)
        .expect_err("operation budget");

    assert_eq!(error.kind, DebugErrorKind::EvaluationLimit);
    assert_eq!(session.output().lines, before);
}

#[test]
fn evaluation_resolves_a_qualified_fieldless_constructor_once() {
    let expression = DebugExpression::Field {
        base: Box::new(DebugExpression::Field {
            base: Box::new(DebugExpression::Field {
                base: Box::new(DebugExpression::Name("Library".to_string())),
                name: "Unit".to_string(),
            }),
            name: "Choice".to_string(),
        }),
        name: "Empty".to_string(),
    };
    let mut calls = 0;

    let value = crate::vm::debug::evaluation::evaluate_value(
        &expression,
        DebugEvaluationLimits::default(),
        |name| {
            Err(crate::DebugSessionError {
                kind: DebugErrorKind::UnknownName,
                message: format!("unknown name `{name}`"),
                hint: "Use a visible binding.".to_string(),
            })
        },
        |target, arguments| {
            calls += 1;
            assert!(arguments.is_empty());
            let crate::vm::debug::evaluation::DebugCallTarget::Named(target) = target else {
                panic!("expected named constructor target")
            };
            assert_eq!(target, "Library.Unit.Choice.Empty");
            Ok(Value::Unit)
        },
    )
    .expect("qualified fieldless constructor");

    assert!(matches!(value, Value::Unit));
    assert_eq!(calls, 1);
}
