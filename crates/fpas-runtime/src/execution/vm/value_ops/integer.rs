//! Integer arithmetic and comparison primitives.
//!
//! Operator semantics: `docs/pascal/language/basics/operators.md`.

use crate::Value;

use super::{BinaryOperation, UnaryOperation, ValueOperationError};

#[inline]
/// Applies numeric negation to an integer.
pub(super) fn unary(operation: UnaryOperation, value: i64) -> Result<Value, ValueOperationError> {
    match operation {
        UnaryOperation::Negate => fpas_numeric::integer::negate(value)
            .map(Value::Integer)
            .map_err(Into::into),
        UnaryOperation::Not => unreachable!("Boolean operation routed to integer value operations"),
    }
}

#[inline]
/// Applies integer arithmetic or comparisons to two integer values.
pub(super) fn binary(
    operation: BinaryOperation,
    left: i64,
    right: i64,
) -> Result<Value, ValueOperationError> {
    match operation {
        BinaryOperation::Add => arithmetic(fpas_numeric::integer::Operation::Add, left, right),
        BinaryOperation::Subtract => {
            arithmetic(fpas_numeric::integer::Operation::Subtract, left, right)
        }
        BinaryOperation::Multiply => {
            arithmetic(fpas_numeric::integer::Operation::Multiply, left, right)
        }
        BinaryOperation::IntegerDivide => {
            arithmetic(fpas_numeric::integer::Operation::Divide, left, right)
        }
        BinaryOperation::Modulo => {
            arithmetic(fpas_numeric::integer::Operation::Remainder, left, right)
        }
        BinaryOperation::Equal => Ok(Value::Boolean(left == right)),
        BinaryOperation::NotEqual => Ok(Value::Boolean(left != right)),
        BinaryOperation::Less => Ok(Value::Boolean(left < right)),
        BinaryOperation::LessEqual => Ok(Value::Boolean(left <= right)),
        BinaryOperation::Greater => Ok(Value::Boolean(left > right)),
        BinaryOperation::GreaterEqual => Ok(Value::Boolean(left >= right)),
        BinaryOperation::RealDivide
        | BinaryOperation::In
        | BinaryOperation::And
        | BinaryOperation::Or
        | BinaryOperation::Xor => {
            unreachable!("non-integer operation routed to integer value operations")
        }
    }
}

fn arithmetic(
    operation: fpas_numeric::integer::Operation,
    left: i64,
    right: i64,
) -> Result<Value, ValueOperationError> {
    fpas_numeric::integer::binary(operation, left, right)
        .map(Value::Integer)
        .map_err(Into::into)
}
