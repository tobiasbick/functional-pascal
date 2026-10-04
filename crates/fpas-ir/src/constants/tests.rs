//! Checked arithmetic boundaries and IEEE folding semantics.

use super::*;

#[test]
fn checked_integer_failures_remain_unfolded() {
    for (operation, left, right, error) in [
        (
            BinaryOperation::AddInteger,
            i64::MAX,
            1,
            ConstantEvaluationError::IntegerOverflow("addition"),
        ),
        (
            BinaryOperation::SubtractInteger,
            i64::MIN,
            1,
            ConstantEvaluationError::IntegerOverflow("subtraction"),
        ),
        (
            BinaryOperation::MultiplyInteger,
            i64::MAX,
            2,
            ConstantEvaluationError::IntegerOverflow("multiplication"),
        ),
        (
            BinaryOperation::DivideInteger,
            i64::MIN,
            -1,
            ConstantEvaluationError::IntegerOverflow("division"),
        ),
        (
            BinaryOperation::RemainderInteger,
            i64::MIN,
            -1,
            ConstantEvaluationError::IntegerOverflow("modulo"),
        ),
        (
            BinaryOperation::DivideInteger,
            1,
            0,
            ConstantEvaluationError::IntegerDivisionByZero,
        ),
        (
            BinaryOperation::RemainderInteger,
            1,
            0,
            ConstantEvaluationError::IntegerRemainderByZero,
        ),
    ] {
        let (left, right) = (Constant::Integer(left), Constant::Integer(right));
        assert_eq!(evaluate_binary(operation, &left, &right), Err(error));
        assert_eq!(fold_binary(operation, &left, &right), None);
    }
    assert_eq!(
        evaluate_unary(UnaryOperation::NegateInteger, &Constant::Integer(i64::MIN)),
        Err(ConstantEvaluationError::IntegerOverflow("negation"))
    );
    assert_eq!(
        fold_unary(UnaryOperation::NegateInteger, &Constant::Integer(i64::MIN)),
        None
    );
}

#[test]
fn integer_endpoints_and_truncating_division_are_representable() {
    for (operation, left, right, expected) in [
        (BinaryOperation::AddInteger, i64::MAX, 0, i64::MAX),
        (BinaryOperation::SubtractInteger, -i64::MAX, 1, i64::MIN),
        (BinaryOperation::MultiplyInteger, i64::MIN, 1, i64::MIN),
        (BinaryOperation::DivideInteger, -7, 3, -2),
        (BinaryOperation::RemainderInteger, -7, 3, -1),
    ] {
        assert_eq!(
            fold_binary(
                operation,
                &Constant::Integer(left),
                &Constant::Integer(right)
            ),
            Some(Constant::Integer(expected))
        );
    }
}

#[test]
fn ieee_division_folds_signed_infinities_and_nan() {
    for (left, right, expected) in [(1.0, 0.0, f64::INFINITY), (1.0, -0.0, f64::NEG_INFINITY)] {
        assert_eq!(
            fold_binary(
                BinaryOperation::DivideReal,
                &Constant::Real(left),
                &Constant::Real(right)
            ),
            Some(Constant::Real(expected))
        );
    }
    let Some(Constant::Real(nan)) = fold_binary(
        BinaryOperation::DivideReal,
        &Constant::Real(0.0),
        &Constant::Real(0.0),
    ) else {
        panic!("expected NaN");
    };
    assert!(nan.is_nan());
    for operation in [
        BinaryOperation::Equal,
        BinaryOperation::LessThanReal,
        BinaryOperation::LessEqualReal,
        BinaryOperation::GreaterThanReal,
        BinaryOperation::GreaterEqualReal,
    ] {
        assert_eq!(
            fold_binary(operation, &Constant::Real(nan), &Constant::Real(1.0)),
            Some(Constant::Boolean(false))
        );
    }
    assert_eq!(
        fold_binary(
            BinaryOperation::NotEqual,
            &Constant::Real(nan),
            &Constant::Real(nan)
        ),
        Some(Constant::Boolean(true))
    );
}

#[test]
fn unsupported_operands_are_distinct_from_checked_failure() {
    assert_eq!(
        evaluate_binary(
            BinaryOperation::AddInteger,
            &Constant::Boolean(true),
            &Constant::Integer(1)
        ),
        Ok(None)
    );
    assert_eq!(
        evaluate_unary(UnaryOperation::NegateInteger, &Constant::Real(1.0)),
        Ok(None)
    );
    assert_eq!(
        fold_binary(
            BinaryOperation::ConcatString,
            &Constant::String("a".into()),
            &Constant::String("b".into())
        ),
        Some(Constant::String("ab".into()))
    );
}
