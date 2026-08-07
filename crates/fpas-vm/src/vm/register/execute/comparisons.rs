//! Typed scalar comparisons and string concatenation.

use std::cmp::Ordering;

use fpas_bytecode::{AbcOperands, SharedStr, Value};

use super::super::VmError;
use super::super::worker::RegisterWorker;
use super::scalar::register;

impl RegisterWorker {
    pub fn execute_compare_integer(
        &mut self,
        operands: AbcOperands,
        compare: impl FnOnce(i64, i64) -> bool,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.integer(register(operands.b)?)?;
        let right = self.integer(register(operands.c)?)?;
        self.write(destination, Value::Boolean(compare(left, right)))
    }

    pub fn execute_compare_real(
        &mut self,
        operands: AbcOperands,
        compare: impl FnOnce(f64, f64) -> bool,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.real(register(operands.b)?)?;
        let right = self.real(register(operands.c)?)?;
        self.write(destination, Value::Boolean(compare(left, right)))
    }

    pub fn execute_compare_boolean(
        &mut self,
        operands: AbcOperands,
        compare: impl FnOnce(bool, bool) -> bool,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = self.boolean(register(operands.b)?)?;
        let right = self.boolean(register(operands.c)?)?;
        self.write(destination, Value::Boolean(compare(left, right)))
    }

    pub fn execute_compare_string(
        &mut self,
        operands: AbcOperands,
        compare: impl FnOnce(Ordering) -> bool,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let result = match (
            self.read(register(operands.b)?)?,
            self.read(register(operands.c)?)?,
        ) {
            (Value::Str(left), Value::Str(right)) => compare(left.cmp(right)),
            (left, _) => {
                return Err(self.runtime_error(
                    fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("Expected string, got {}", left.type_name()),
                    "Use string operands for this VM operation.",
                ));
            }
        };
        self.write(destination, Value::Boolean(result))
    }

    pub fn execute_concat_string(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let result = match (
            self.read(register(operands.b)?)?,
            self.read(register(operands.c)?)?,
        ) {
            (Value::Str(left), Value::Str(right)) => SharedStr::concat(left, right),
            (left, _) => {
                return Err(self.runtime_error(
                    fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("Expected string, got {}", left.type_name()),
                    "Use string operands when concatenating values.",
                ));
            }
        };
        self.write(destination, Value::Str(result))
    }
}
