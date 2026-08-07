//! Central checked register and persistent-constant access.

use fpas_bytecode::{Constant, Register, SharedStr, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

use super::VmError;
use super::diagnostics;
use super::worker::RegisterWorker;

impl RegisterWorker {
    pub fn read(&self, register: Register) -> Result<&Value, VmError> {
        self.registers
            .get(self.base + usize::from(register.get()))
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

    pub fn write(&mut self, register: Register, value: Value) -> Result<(), VmError> {
        let executable = self.executable.executable();
        let address = self.current_address;
        let slot = self
            .registers
            .get_mut(self.base + usize::from(register.get()))
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

    pub fn integer(&self, register: Register) -> Result<i64, VmError> {
        match self.read(register)? {
            Value::Integer(value) => Ok(*value),
            Value::Boolean(value) => Ok(i64::from(*value)),
            other => Err(self.type_error("integer-compatible", other)),
        }
    }

    pub fn real(&self, register: Register) -> Result<f64, VmError> {
        match self.read(register)? {
            Value::Real(value) => Ok(*value),
            other => Err(self.type_error("real", other)),
        }
    }

    pub fn boolean(&self, register: Register) -> Result<bool, VmError> {
        match self.read(register)? {
            Value::Boolean(value) => Ok(*value),
            other => Err(self.type_error("boolean", other)),
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
                Ok(Value::register_function(
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
