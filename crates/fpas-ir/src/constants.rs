//! Shared scalar constant evaluation for semantic interfaces and compiler optimization.
//!
//! Integer arithmetic is checked. Failed folds preserve runtime operations, while
//! static evaluation exposes the same failure for semantic diagnostics.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md`.

mod error;
#[cfg(test)]
mod tests;

pub use error::ConstantEvaluationError;

use crate::{BinaryOperation, Constant, UnaryOperation};

/// Evaluate one binary operation, distinguishing failure from unsupported operands.
pub fn evaluate_binary(
    operation: BinaryOperation,
    left: &Constant,
    right: &Constant,
) -> Result<Option<Constant>, ConstantEvaluationError> {
    use BinaryOperation as Op;
    use Constant::{Boolean, Integer, Real, String};
    Ok(Some(match (operation, left, right) {
        (Op::AddInteger, Integer(left), Integer(right)) => Integer(
            left.checked_add(*right)
                .ok_or(ConstantEvaluationError::IntegerOverflow("addition"))?,
        ),
        (Op::SubtractInteger, Integer(left), Integer(right)) => Integer(
            left.checked_sub(*right)
                .ok_or(ConstantEvaluationError::IntegerOverflow("subtraction"))?,
        ),
        (Op::MultiplyInteger, Integer(left), Integer(right)) => Integer(
            left.checked_mul(*right)
                .ok_or(ConstantEvaluationError::IntegerOverflow("multiplication"))?,
        ),
        (Op::DivideInteger, Integer(left), Integer(right)) => {
            if *right == 0 {
                return Err(ConstantEvaluationError::IntegerDivisionByZero);
            }
            Integer(
                left.checked_div(*right)
                    .ok_or(ConstantEvaluationError::IntegerOverflow("division"))?,
            )
        }
        (Op::RemainderInteger, Integer(left), Integer(right)) => {
            if *right == 0 {
                return Err(ConstantEvaluationError::IntegerRemainderByZero);
            }
            Integer(
                left.checked_rem(*right)
                    .ok_or(ConstantEvaluationError::IntegerOverflow("modulo"))?,
            )
        }
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
        (Op::DivideReal, Real(left), Real(right)) => Real(left / right),
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
                _ => return Ok(None),
            };
            Boolean(equal == (operation == Op::Equal))
        }
        (Op::AndBoolean, Boolean(left), Boolean(right)) => Boolean(*left && *right),
        (Op::OrBoolean, Boolean(left), Boolean(right)) => Boolean(*left || *right),
        (Op::ConcatString, String(left), String(right)) => String(format!("{left}{right}")),
        _ => return Ok(None),
    }))
}

/// Evaluate one unary operation, distinguishing failure from unsupported operands.
pub fn evaluate_unary(
    operation: UnaryOperation,
    operand: &Constant,
) -> Result<Option<Constant>, ConstantEvaluationError> {
    Ok(Some(match (operation, operand) {
        (UnaryOperation::NegateInteger, Constant::Integer(value)) => Constant::Integer(
            value
                .checked_neg()
                .ok_or(ConstantEvaluationError::IntegerOverflow("negation"))?,
        ),
        (UnaryOperation::NegateReal, Constant::Real(value)) => Constant::Real(-value),
        (UnaryOperation::NotBoolean, Constant::Boolean(value)) => Constant::Boolean(!value),
        (UnaryOperation::IntegerToReal, Constant::Integer(value)) => Constant::Real(*value as f64),
        _ => return Ok(None),
    }))
}

/// Fold a binary operation only when checked evaluation succeeds.
///
/// A failing operation remains executable so optimization preserves its panic.
pub fn fold_binary(
    operation: BinaryOperation,
    left: &Constant,
    right: &Constant,
) -> Option<Constant> {
    evaluate_binary(operation, left, right).ok().flatten()
}

/// Fold a unary operation only when checked evaluation succeeds.
pub fn fold_unary(operation: UnaryOperation, operand: &Constant) -> Option<Constant> {
    evaluate_unary(operation, operand).ok().flatten()
}
