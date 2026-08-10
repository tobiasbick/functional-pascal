//! Scalar and conversion handlers backed by shared pure value operations.

use fpas_bytecode::{AbcOperands, Register, Value};

use super::super::value_ops::{self, BinaryOperation, UnaryOperation, ValueOperationError};
use super::super::worker::Worker;
use super::super::{VmError, diagnostics};

impl Worker {
    pub fn execute_value_binary(
        &mut self,
        operands: AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.read(register(operands.b)?)?.clone();
        let right = self.read(register(operands.c)?)?.clone();
        let result = value_ops::binary(operation, &left, &right)
            .map_err(|error| self.value_operation_error(error))?;
        self.write(destination, result)
    }

    pub fn execute_value_unary(
        &mut self,
        operands: AbcOperands,
        operation: UnaryOperation,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let value = self.read(register(operands.b)?)?.clone();
        let result = value_ops::unary(operation, &value)
            .map_err(|error| self.value_operation_error(error))?;
        self.write(destination, result)
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
pub(in crate::vm) fn register(value: u16) -> Result<Register, VmError> {
    Register::new(value).map_err(|error| {
        fpas_diagnostics::Diagnostic::error(
            fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
            format!("Invalid decoded register: {error}"),
            Some("This indicates malformed bytecode passed verifier admission.".to_string()),
            fpas_diagnostics::SourceSpan::new(0, 1, 1, 1),
        )
    })
}
