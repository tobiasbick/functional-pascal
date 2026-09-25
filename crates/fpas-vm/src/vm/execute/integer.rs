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
