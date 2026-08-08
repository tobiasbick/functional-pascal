//! Integer, real, boolean, and conversion handlers.

use fpas_bytecode::{AbcOperands, Register, Value};
use fpas_diagnostics::codes::{
    RUNTIME_DIVISION_BY_ZERO, RUNTIME_MODULO_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR,
};

use super::super::worker::Worker;
use super::super::{VmError, diagnostics};

impl Worker {
    pub fn execute_binary_integer(
        &mut self,
        operands: AbcOperands,
        operation: impl FnOnce(i64, i64) -> i64,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.integer(register(operands.b)?)?;
        let right = self.integer(register(operands.c)?)?;
        self.write(destination, Value::Integer(operation(left, right)))
    }

    pub fn execute_divide_integer(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.integer(register(operands.b)?)?;
        let right = self.integer(register(operands.c)?)?;
        if right == 0 {
            return Err(self.runtime_error(
                RUNTIME_DIVISION_BY_ZERO,
                "Division by zero",
                "Check the right-hand side before using `div` or `/`.",
            ));
        }
        let value = left.checked_div(right).ok_or_else(|| {
            self.runtime_error(
                RUNTIME_NUMERIC_DOMAIN_ERROR,
                "Integer division overflow",
                "Avoid dividing the minimum integer value by `-1`.",
            )
        })?;
        self.write(destination, Value::Integer(value))
    }

    pub fn execute_remainder_integer(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.integer(register(operands.b)?)?;
        let right = self.integer(register(operands.c)?)?;
        if right == 0 {
            return Err(self.runtime_error(
                RUNTIME_MODULO_BY_ZERO,
                "Modulo by zero",
                "Check the right-hand side before using `mod`.",
            ));
        }
        let value = left.checked_rem(right).ok_or_else(|| {
            self.runtime_error(
                RUNTIME_NUMERIC_DOMAIN_ERROR,
                "Integer modulo overflow",
                "Avoid applying `mod` with the minimum integer value and `-1`.",
            )
        })?;
        self.write(destination, Value::Integer(value))
    }

    pub fn execute_shift_integer(
        &mut self,
        operands: AbcOperands,
        left_shift: bool,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let value = self.integer(register(operands.b)?)?;
        let amount = self.integer(register(operands.c)?)?;
        let amount = u32::try_from(amount)
            .ok()
            .filter(|amount| *amount < 64)
            .ok_or_else(|| {
                self.runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    format!("Shift amount {amount} is out of range (0..63)"),
                    "Use a shift amount between 0 and 63 inclusive.",
                )
            })?;
        let result = if left_shift {
            value.wrapping_shl(amount)
        } else {
            value.wrapping_shr(amount)
        };
        self.write(destination, Value::Integer(result))
    }

    pub fn execute_binary_real(
        &mut self,
        operands: AbcOperands,
        operation: impl FnOnce(f64, f64) -> f64,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.real(register(operands.b)?)?;
        let right = self.real(register(operands.c)?)?;
        self.write(destination, Value::Real(operation(left, right)))
    }

    pub fn execute_divide_real(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.real(register(operands.b)?)?;
        let right = self.real(register(operands.c)?)?;
        if right == 0.0 {
            return Err(self.runtime_error(
                RUNTIME_DIVISION_BY_ZERO,
                "Division by zero",
                "Check the right-hand side before using `/`.",
            ));
        }
        self.write(destination, Value::Real(left / right))
    }

    pub fn execute_negate_integer(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let value = self.integer(register(operands.b)?)?;
        let value = value.checked_neg().ok_or_else(|| {
            self.runtime_error(
                RUNTIME_NUMERIC_DOMAIN_ERROR,
                "Integer negation overflow",
                "Avoid negating the minimum integer value.",
            )
        })?;
        self.write(destination, Value::Integer(value))
    }

    pub fn execute_negate_real(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let value = self.real(register(operands.b)?)?;
        self.write(destination, Value::Real(-value))
    }

    pub fn execute_not_boolean(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let value = self.boolean(register(operands.b)?)?;
        self.write(destination, Value::Boolean(!value))
    }

    pub fn execute_integer_to_real(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let value = self.integer(register(operands.b)?)?;
        self.write(destination, Value::Real(value as f64))
    }

    pub fn execute_binary_boolean(
        &mut self,
        operands: AbcOperands,
        operation: impl FnOnce(bool, bool) -> bool,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.boolean(register(operands.b)?)?;
        let right = self.boolean(register(operands.c)?)?;
        self.write(destination, Value::Boolean(operation(left, right)))
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
