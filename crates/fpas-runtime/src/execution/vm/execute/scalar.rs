//! Scalar and conversion handlers backed by shared pure value operations.

use crate::{AbcOperands, Register, Value};

use super::super::value_ops::{self, BinaryOperation, UnaryOperation, ValueOperationError};
use super::super::worker::Worker;
use super::super::{VmError, diagnostics};
use super::integer::direct_integer_binary;

impl Worker {
    pub fn execute_value_binary(
        &mut self,
        operands: AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let left = self.read_operand(operands.b)?;
        let right = self.read_operand(operands.c)?;
        if let (Value::Integer(left), Value::Integer(right)) = (left, right)
            && let Some(result) = direct_integer_binary(operation, *left, *right)
        {
            return self.write_direct_integer_result(operands.a, result);
        }
        let result = value_ops::binary(operation, left, right)
            .map_err(|error| self.value_operation_error(error))?;
        self.write_operand(operands.a, result)
    }

    pub fn execute_value_unary(
        &mut self,
        operands: AbcOperands,
        operation: UnaryOperation,
    ) -> Result<(), VmError> {
        let value = self.read_operand(operands.b)?;
        if let (UnaryOperation::Not, Value::Boolean(value)) = (operation, value) {
            return self.write_boolean_operand(operands.a, !value);
        }
        let result = value_ops::unary(operation, value)
            .map_err(|error| self.value_operation_error(error))?;
        self.write_operand(operands.a, result)
    }

    pub fn execute_integer_to_real(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let value = self.integer(register(operands.b)?)?;
        self.write(destination, Value::Real(value as f64))
    }

    fn value_operation_error(&self, error: ValueOperationError) -> VmError {
        self.runtime_error(error.code, error.message, error.hint)
    }

    pub fn runtime_error(
        &self,
        code: fpas_diagnostics::DiagnosticCode,
        message: impl Into<String>,
        help: impl Into<String>,
    ) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            code,
            message,
            help,
        )
    }
}

#[inline(always)]
pub(in crate::execution::vm) fn register(value: u16) -> Result<Register, VmError> {
    Register::new(value).map_err(|error| {
        Box::new(fpas_diagnostics::Diagnostic::error_without_source(
            fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
            format!("Invalid decoded register: {error}"),
            Some("This indicates malformed bytecode passed verifier admission.".to_string()),
        ))
    })
}
