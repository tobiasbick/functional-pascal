//! Central checked register and persistent-constant access.

use fpas_bytecode::{Constant, NO_REGISTER, Register, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

use super::VmError;
use super::diagnostics;
use super::worker::Worker;

impl Worker {
    /// Borrow a register of the current frame.
    #[inline(always)]
    pub fn read(&self, register: Register) -> Result<&Value, VmError> {
        self.read_operand(register.get())
    }

    /// Borrow a verified register operand of the current frame.
    ///
    /// The register vector holds exactly the active window and the verifier keeps every register
    /// operand below the frame's register count, so one length check covers the frame bound.
    #[inline(always)]
    pub(super) fn read_operand(&self, operand: u16) -> Result<&Value, VmError> {
        debug_assert_ne!(
            operand, NO_REGISTER,
            "verified register operands are never NO_REGISTER"
        );
        let index = self.base + usize::from(operand);
        match self.registers.get(index) {
            Some(value) => Ok(value),
            None => Err(self.register_outside_frame(index)),
        }
    }

    /// Replace a register of the current frame and mark it initialized.
    #[inline(always)]
    pub fn write(&mut self, register: Register, value: Value) -> Result<(), VmError> {
        self.write_operand(register.get(), value)
    }

    /// Replace a verified register operand of the current frame and mark it initialized.
    #[inline(always)]
    pub(super) fn write_operand(&mut self, operand: u16, value: Value) -> Result<(), VmError> {
        debug_assert_ne!(
            operand, NO_REGISTER,
            "verified register operands are never NO_REGISTER"
        );
        self.store_register(self.base + usize::from(operand), value)
    }

    /// Remove a value from a register without cloning it.
    pub(super) fn take(&mut self, register: Register) -> Result<Value, VmError> {
        self.take_register(self.base + usize::from(register.get()))
    }

    #[inline(always)]
    pub fn integer(&self, register: Register) -> Result<i64, VmError> {
        match self.read(register)? {
            Value::Integer(value) => Ok(*value),
            Value::Boolean(value) => Ok(i64::from(*value)),
            other => Err(self.type_error("integer-compatible", other)),
        }
    }

    pub fn load_constant(&self, index: u32) -> Result<Value, VmError> {
        let executable = self.executable.executable();
        let constant = usize::try_from(index)
            .ok()
            .and_then(|index| executable.constants.get(index))
            .ok_or_else(|| {
                diagnostics::internal(
                    executable,
                    self.current_address,
                    format!("Constant {index} is outside the verified constant table"),
                )
            })?;
        match *constant {
            Constant::Integer(value) => Ok(Value::Integer(value)),
            Constant::Real(bits) => Ok(Value::Real(f64::from_bits(bits))),
            Constant::Boolean(value) => Ok(Value::Boolean(value)),
            Constant::Unit => Ok(Value::Unit),
            Constant::String(string) => self
                .executable
                .string_constant(index as usize)
                .cloned()
                .map(Value::Str)
                .ok_or_else(|| {
                    diagnostics::internal(
                        executable,
                        self.current_address,
                        format!(
                            "String {} is outside the verified string table",
                            string.get()
                        ),
                    )
                }),
            Constant::Function {
                function,
                task_bound,
            } => {
                let info = executable
                    .functions
                    .get(usize::from(function.get()))
                    .ok_or_else(|| {
                        diagnostics::internal(
                            executable,
                            self.current_address,
                            "Function constant target is missing",
                        )
                    })?;
                let name = executable.strings.get(info.name).ok_or_else(|| {
                    diagnostics::internal(
                        executable,
                        self.current_address,
                        "Function constant name is missing",
                    )
                })?;
                Ok(if task_bound {
                    Value::task_owned_function(function, name.to_owned(), Vec::new(), self.task_id)
                } else {
                    Value::function(function, name.to_owned(), Vec::new())
                })
            }
        }
    }

    fn type_error(&self, expected: &str, actual: &Value) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected {expected}, got {}", actual.type_name()),
            format!("Use {expected} operands for this VM operation."),
        )
    }
}
