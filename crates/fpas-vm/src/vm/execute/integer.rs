//! Direct handlers for statically typed integer bytecode.

use fpas_bytecode::{AbcOperands, Value};

use super::super::VmError;
use super::super::value_ops::{self, BinaryOperation, UnaryOperation};
use super::super::worker::Worker;
use super::scalar::register;

impl Worker {
    /// Execute a typed integer binary operation while preserving malformed-bytecode diagnostics.
    #[inline]
    pub(in crate::vm) fn execute_integer_binary(
        &mut self,
        operands: AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.read(register(operands.b)?)?;
        let right = self.read(register(operands.c)?)?;
        let result = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                value_ops::integer_binary(operation, *left, *right)
            }
            _ => value_ops::binary(operation, left, right),
        }
        .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?;
        self.write(destination, result)
    }

    /// Execute a typed integer unary operation while preserving malformed-bytecode diagnostics.
    #[inline]
    pub(in crate::vm) fn execute_integer_unary(
        &mut self,
        operands: AbcOperands,
        operation: UnaryOperation,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let value = self.read(register(operands.b)?)?;
        let result = match value {
            Value::Integer(value) => value_ops::integer_unary(operation, *value),
            _ => value_ops::unary(operation, value),
        }
        .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?;
        self.write(destination, result)
    }
}
