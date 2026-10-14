//! Direct handlers for statically typed integer bytecode.

use fpas_bytecode::{AbcOperands, Value};

use super::super::VmError;
use super::super::value_ops::{self, BinaryOperation, UnaryOperation};
use super::super::worker::Worker;

impl Worker {
    /// Execute a typed integer binary operation on verified register operands.
    #[inline(always)]
    pub(in crate::vm) fn execute_integer_binary(
        &mut self,
        operands: AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let left = self.read_operand(operands.b)?;
        let right = self.read_operand(operands.c)?;
        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                if let Some(result) = direct_integer_binary(operation, *left, *right) {
                    return self.write_operand(operands.a, result);
                }
                value_ops::integer_binary(operation, *left, *right)
            }
            _ => value_ops::binary(operation, left, right),
        }
        .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?;
        self.write_operand(operands.a, result)
    }

    /// Execute a typed integer unary operation on verified register operands.
    #[inline(always)]
    pub(in crate::vm) fn execute_integer_unary(
        &mut self,
        operands: AbcOperands,
        operation: UnaryOperation,
    ) -> Result<(), VmError> {
        let value = self.read_operand(operands.b)?;
        let result = match value {
            Value::Integer(value) => value_ops::integer_unary(operation, *value),
            _ => value_ops::unary(operation, value),
        }
        .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?;
        self.write_operand(operands.a, result)
    }
}

/// Computes in-range integer arithmetic and comparisons without building a diagnostic result.
///
/// Returns `None` for overflow, zero divisors, and other operations; the checked path in
/// `value_ops` then reports the documented error. Semantics:
/// `docs/pascal/language/basics/operators.md`.
#[inline]
pub(in crate::vm) fn direct_integer_binary(
    operation: BinaryOperation,
    left: i64,
    right: i64,
) -> Option<Value> {
    Some(match operation {
        BinaryOperation::Add => Value::Integer(left.checked_add(right)?),
        BinaryOperation::Subtract => Value::Integer(left.checked_sub(right)?),
        BinaryOperation::Multiply => Value::Integer(left.checked_mul(right)?),
        BinaryOperation::IntegerDivide => Value::Integer(left.checked_div(right)?),
        BinaryOperation::Modulo => Value::Integer(left.checked_rem(right)?),
        BinaryOperation::Equal => Value::Boolean(left == right),
        BinaryOperation::NotEqual => Value::Boolean(left != right),
        BinaryOperation::Less => Value::Boolean(left < right),
        BinaryOperation::LessEqual => Value::Boolean(left <= right),
        BinaryOperation::Greater => Value::Boolean(left > right),
        BinaryOperation::GreaterEqual => Value::Boolean(left >= right),
        _ => return None,
    })
}
