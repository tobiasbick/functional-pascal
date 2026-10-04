//! Shared scalar constant evaluation for semantic interfaces and compiler optimization.
//!
//! Integer addition, subtraction and multiplication wrap like the current runtime.
//! Failing operations remain unevaluated so callers preserve their runtime errors.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md`.

use crate::{BinaryOperation, Constant, UnaryOperation};

/// Evaluate a scalar binary operation; return `None` when it would fail at runtime.
pub fn fold_binary(
    operation: BinaryOperation,
    left: &Constant,
    right: &Constant,
) -> Option<Constant> {
    use BinaryOperation as Op;
    use Constant::{Boolean, Integer, Real, String};
    Some(match (operation, left, right) {
        (Op::AddInteger, Integer(left), Integer(right)) => Integer(left.wrapping_add(*right)),
        (Op::SubtractInteger, Integer(left), Integer(right)) => Integer(left.wrapping_sub(*right)),
        (Op::MultiplyInteger, Integer(left), Integer(right)) => Integer(left.wrapping_mul(*right)),
        (Op::DivideInteger, Integer(left), Integer(right)) => Integer(left.checked_div(*right)?),
        (Op::RemainderInteger, Integer(left), Integer(right)) => Integer(left.checked_rem(*right)?),
        (Op::BitAndInteger, Integer(left), Integer(right)) => Integer(left & right),
        (Op::BitOrInteger, Integer(left), Integer(right)) => Integer(left | right),
        (Op::BitXorInteger, Integer(left), Integer(right)) => Integer(left ^ right),
        (Op::LessThanInteger, Integer(left), Integer(right)) => Boolean(left < right),
        (Op::GreaterThanInteger, Integer(left), Integer(right)) => Boolean(left > right),
        (Op::LessEqualInteger, Integer(left), Integer(right)) => Boolean(left <= right),
        (Op::GreaterEqualInteger, Integer(left), Integer(right)) => Boolean(left >= right),
        (Op::AddReal, Real(left), Real(right)) => Real(left + right),
        (Op::SubtractReal, Real(left), Real(right)) => Real(left - right),
        (Op::MultiplyReal, Real(left), Real(right)) => Real(left * right),
        (Op::DivideReal, Real(left), Real(right)) if *right != 0.0 => Real(left / right),
        (Op::LessThanReal, Real(left), Real(right)) => Boolean(left < right),
        (Op::GreaterThanReal, Real(left), Real(right)) => Boolean(left > right),
        (Op::LessEqualReal, Real(left), Real(right)) => Boolean(left <= right),
        (Op::GreaterEqualReal, Real(left), Real(right)) => Boolean(left >= right),
        (Op::LessThanString, String(left), String(right)) => Boolean(left < right),
        (Op::GreaterThanString, String(left), String(right)) => Boolean(left > right),
        (Op::LessEqualString, String(left), String(right)) => Boolean(left <= right),
        (Op::GreaterEqualString, String(left), String(right)) => Boolean(left >= right),
        (Op::Equal | Op::NotEqual, left, right) => {
            let equal = match (left, right) {
                (Integer(left), Integer(right)) => left == right,
                (Real(left), Real(right)) => left == right,
                (Boolean(left), Boolean(right)) => left == right,
                (String(left), String(right)) => left == right,
                _ => return None,
            };
            Boolean(equal == (operation == Op::Equal))
        }
        (Op::AndBoolean, Boolean(left), Boolean(right)) => Boolean(*left && *right),
        (Op::OrBoolean, Boolean(left), Boolean(right)) => Boolean(*left || *right),
        (Op::ConcatString, String(left), String(right)) => String(format!("{left}{right}")),
        _ => return None,
    })
}

/// Evaluate a scalar unary operation; return `None` when it would fail at runtime.
pub fn fold_unary(operation: UnaryOperation, operand: &Constant) -> Option<Constant> {
    Some(match (operation, operand) {
        (UnaryOperation::NegateInteger, Constant::Integer(value)) => {
            Constant::Integer(value.checked_neg()?)
        }
        (UnaryOperation::NegateReal, Constant::Real(value)) => Constant::Real(-value),
        (UnaryOperation::NotBoolean, Constant::Boolean(value)) => Constant::Boolean(!value),
        (UnaryOperation::IntegerToReal, Constant::Integer(value)) => Constant::Real(*value as f64),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_matches_runtime_semantics_and_keeps_failing_operations() {
        use Constant::{Boolean, Integer, Real, String};
        assert_eq!(
            fold_binary(BinaryOperation::AddInteger, &Integer(i64::MAX), &Integer(1)),
            Some(Integer(i64::MIN)),
            "integer addition wraps like the VM"
        );
        assert_eq!(
            fold_binary(BinaryOperation::DivideInteger, &Integer(7), &Integer(0)),
            None
        );
        assert_eq!(
            fold_binary(
                BinaryOperation::DivideInteger,
                &Integer(i64::MIN),
                &Integer(-1)
            ),
            None
        );
        assert_eq!(
            fold_binary(BinaryOperation::RemainderInteger, &Integer(-7), &Integer(2)),
            Some(Integer(-1))
        );
        assert_eq!(
            fold_binary(BinaryOperation::DivideReal, &Real(1.0), &Real(0.0)),
            None
        );
        assert_eq!(
            fold_binary(
                BinaryOperation::ConcatString,
                &String("a".to_string()),
                &String("b".to_string())
            ),
            Some(String("ab".to_string()))
        );
        assert_eq!(
            fold_binary(BinaryOperation::Equal, &Integer(1), &Real(1.0)),
            None
        );
        assert_eq!(
            fold_unary(UnaryOperation::NegateInteger, &Integer(i64::MIN)),
            None
        );
        assert_eq!(
            fold_unary(UnaryOperation::NotBoolean, &Boolean(true)),
            Some(Boolean(false))
        );
    }
}
