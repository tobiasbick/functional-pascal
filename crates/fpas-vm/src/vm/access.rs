//! Central checked register and persistent-constant access.

use fpas_bytecode::{Constant, FunctionId, FunctionTarget, NO_REGISTER, Register, Value};
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

    /// Store a real result in a verified register operand of the current frame.
    #[inline(always)]
    pub(super) fn write_real_operand(&mut self, operand: u16, value: f64) -> Result<(), VmError> {
        debug_assert_ne!(
            operand, NO_REGISTER,
            "verified register operands are never NO_REGISTER"
        );
        self.store_real_register(self.base + usize::from(operand), value)
    }

    /// Store an integer result in a verified register operand of the current frame.
    #[inline(always)]
    pub(super) fn write_integer_operand(
        &mut self,
        operand: u16,
        value: i64,
    ) -> Result<(), VmError> {
        debug_assert_ne!(
            operand, NO_REGISTER,
            "verified register operands are never NO_REGISTER"
        );
        self.store_integer_register(self.base + usize::from(operand), value)
    }

    /// Store a Boolean result in a verified register operand of the current frame.
    #[inline(always)]
    pub(super) fn write_boolean_operand(
        &mut self,
        operand: u16,
        value: bool,
    ) -> Result<(), VmError> {
        debug_assert_ne!(
            operand, NO_REGISTER,
            "verified register operands are never NO_REGISTER"
        );
        self.store_boolean_register(self.base + usize::from(operand), value)
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
                let target = self.function_target(function)?;
                Ok(if task_bound {
                    Value::task_owned_function(target, Vec::new(), self.domain.id().clone())
                } else {
                    Value::function(target, Vec::new())
                })
            }
        }
    }

    /// Mint a callable target from this worker's verified executable.
    pub(in crate::vm) fn function_target(
        &self,
        function: FunctionId,
    ) -> Result<FunctionTarget, VmError> {
        self.executable.function_target(function).ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Function value target is outside the verified function table",
            )
        })
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
