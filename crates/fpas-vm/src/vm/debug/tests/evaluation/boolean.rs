//! Operand visitation and resource budgets for debugger Boolean expressions.

use super::*;
use crate::vm::debug::evaluation::evaluate_value;

#[test]
fn boolean_operands_are_visited_in_order_exactly_once() {
    use DebugBinaryOperation::{And, Or, Xor};
    for (operation, left, right, expected, expected_trace) in [
        (
            And,
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Boolean(false),
            vec!["left"],
        ),
        (
            Or,
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Boolean(true),
            vec!["left"],
        ),
        (
            And,
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Boolean(false),
            vec!["left", "right"],
        ),
        (
            Or,
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Boolean(true),
            vec!["left", "right"],
        ),
        (
            Xor,
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Boolean(true),
            vec!["left", "right"],
        ),
        (
            Xor,
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(false),
            vec!["left", "right"],
        ),
    ] {
        let expression = DebugExpression::Binary {
            operation,
            left: Box::new(DebugExpression::Callable("left".to_string())),
            right: Box::new(DebugExpression::Callable("right".to_string())),
        };
        let mut trace = Vec::new();
        let result = evaluate_value(
            &expression,
            DebugEvaluationLimits::default(),
            |_| panic!("no names to resolve"),
            |target, arguments| {
                let crate::vm::debug::evaluation::DebugCallTarget::Named(name) = target else {
                    panic!("expected named call")
                };
                assert!(arguments.is_empty());
                trace.push(name.clone());
                Ok(if name == "left" {
                    left.clone()
                } else {
                    right.clone()
                })
            },
        )
        .expect("evaluate Boolean operands");
        assert_eq!(result, expected);
        assert_eq!(trace, expected_trace);
    }
}

#[test]
fn skipped_operand_does_not_consume_the_runtime_evaluation_budget() {
    let expression = DebugExpression::Binary {
        operation: DebugBinaryOperation::And,
        left: Box::new(DebugExpression::Boolean(false)),
        right: Box::new(DebugExpression::Name("unavailable".to_string())),
    };
    let result = evaluate_value(
        &expression,
        DebugEvaluationLimits {
            max_operations: 2,
            ..DebugEvaluationLimits::default()
        },
        |_| panic!("skipped operand must not be resolved"),
        |_, _| panic!("no calls"),
    )
    .expect("budget covers only the operation and left operand");
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn resolved_integer_logical_values_are_rejected_with_bits_hints() {
    for (operation, function) in [
        (DebugBinaryOperation::And, "BitAnd"),
        (DebugBinaryOperation::Or, "BitOr"),
        (DebugBinaryOperation::Xor, "BitXor"),
    ] {
        let expression = DebugExpression::Binary {
            operation,
            left: Box::new(DebugExpression::Name("left".to_string())),
            right: Box::new(DebugExpression::Name("right".to_string())),
        };
        let error = evaluate_value(
            &expression,
            DebugEvaluationLimits::default(),
            |_| Ok(Value::Integer(1)),
            |_, _| panic!("no calls"),
        )
        .expect_err("integer logical operands");
        assert_eq!(error.kind, DebugErrorKind::EvaluationType);
        assert!(error.hint.contains(function), "{error:?}");
        assert!(error.hint.contains("uses Std.Bits;"));
    }
}
