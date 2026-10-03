use super::*;

#[test]
fn debugger_short_circuits_boolean_operands_but_evaluates_xor() {
    for (op, left, calls) in [
        (BinaryOperation::And, false, 0),
        (BinaryOperation::And, true, 1),
        (BinaryOperation::Or, true, 0),
        (BinaryOperation::Or, false, 1),
        (BinaryOperation::Xor, true, 1),
    ] {
        let expression = DebugExpression::Binary {
            operation: op,
            left: Box::new(DebugExpression::Boolean(left)),
            right: Box::new(DebugExpression::Callable("Next".into())),
        };
        let mut count = 0;
        evaluate(
            &expression,
            0,
            DebugEvaluationLimits::default(),
            &mut EvaluationBudget::new(),
            &mut |_| unreachable!(),
            &mut |_, _| {
                count += 1;
                Ok(Value::Boolean(true))
            },
        )
        .unwrap();
        assert_eq!(count, calls);
    }
}

#[test]
fn debugger_rejects_integer_logical_operators() {
    for op in [
        BinaryOperation::And,
        BinaryOperation::Or,
        BinaryOperation::Xor,
    ] {
        let expression = DebugExpression::Binary {
            operation: op,
            left: Box::new(DebugExpression::Integer(1)),
            right: Box::new(DebugExpression::Integer(2)),
        };
        let error = evaluate(
            &expression,
            0,
            DebugEvaluationLimits::default(),
            &mut EvaluationBudget::new(),
            &mut |_| unreachable!(),
            &mut |_, _| unreachable!(),
        )
        .unwrap_err();
        assert_eq!(error.kind, DebugErrorKind::EvaluationType);
    }
}
