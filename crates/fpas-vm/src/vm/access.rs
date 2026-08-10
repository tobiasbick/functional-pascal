//! Central checked register and persistent-constant access.

use std::mem;

use fpas_bytecode::{Constant, Register, SharedStr, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

use super::VmError;
use super::diagnostics;
use super::worker::Worker;

impl Worker {
    #[inline(always)]
    pub fn read(&self, register: Register) -> Result<&Value, VmError> {
        let index = self.base + usize::from(register.get());
        self.registers
            .get(..self.active_register_count)
            .and_then(|registers| registers.get(index))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    format!(
                        "Register {} is outside the initialized frame",
                        register.get()
                    ),
                )
            })
    }

    #[inline(always)]
    pub fn write(&mut self, register: Register, value: Value) -> Result<(), VmError> {
        let executable = self.executable.executable();
        let address = self.current_address;
        let index = self.base + usize::from(register.get());
        let slot = self
            .registers
            .get_mut(..self.active_register_count)
            .and_then(|registers| registers.get_mut(index))
            .ok_or_else(|| {
                diagnostics::internal(
                    executable,
                    address,
                    format!(
                        "Register {} is outside the initialized frame",
                        register.get()
                    ),
                )
            })?;
        *slot = value;
        Ok(())
    }

    /// Remove a value from a register without cloning it.
    pub(super) fn take(&mut self, register: Register) -> Result<Value, VmError> {
        let executable = self.executable.executable();
        let address = self.current_address;
        let index = self.base + usize::from(register.get());
        let slot = self
            .registers
            .get_mut(..self.active_register_count)
            .and_then(|registers| registers.get_mut(index))
            .ok_or_else(|| {
                diagnostics::internal(
                    executable,
                    address,
                    format!(
                        "Register {} is outside the initialized frame",
                        register.get()
                    ),
                )
            })?;
        Ok(mem::replace(slot, Value::Unit))
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
            Constant::String(string) => executable
                .strings
                .get(string)
                .map(|value| Value::Str(SharedStr::from(value)))
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
                Ok(Value::function(
                    function,
                    name.to_owned(),
                    Vec::new(),
                    task_bound,
                ))
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
