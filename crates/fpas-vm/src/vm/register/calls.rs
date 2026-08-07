//! Numeric calls, frame transitions, closures, and mutable capture cells.

use std::sync::{Arc, Mutex};

use fpas_bytecode::{AbcOperands, FunctionId, Register, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::dispatch::DispatchStep;
use super::frame::{CallFrame, MAX_CALL_DEPTH, MAX_REGISTER_SLOTS};
use super::worker::RegisterWorker;
use super::{VmError, diagnostics};

impl RegisterWorker {
    pub(super) fn call_direct(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let target = FunctionId::new(operands.b);
        let arguments = self.clone_window(operands.c, operands.auxiliary)?;
        self.enter_call(target, operands.a, arguments, Vec::new())
    }

    pub(super) fn call_value(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let callee = self.read(self.call_register(operands.b)?)?.clone();
        let Value::Function(function) = callee else {
            return Err(self.operand_type_error("function", &callee));
        };
        let target = function.function.ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "A legacy name-only function value reached the numeric register call path",
            )
        })?;
        let arguments = self.clone_window(operands.c, operands.auxiliary)?;
        self.enter_call(target, operands.a, arguments, function.captures.clone())
    }

    pub(super) fn make_closure(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let target = FunctionId::new(operands.b);
        let captures = self.clone_window(operands.c, operands.auxiliary)?;
        let task_bound = captures.iter().any(|capture| match capture {
            Value::Cell(_) => true,
            Value::Function(function) => function.task_bound,
            _ => false,
        });
        let image = self.executable.executable();
        let info = image
            .functions
            .get(usize::from(target.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    image,
                    self.current_address,
                    "Closure target is outside the function table",
                )
            })?;
        let name = image.strings.get(info.name).ok_or_else(|| {
            diagnostics::internal(
                image,
                self.current_address,
                "Closure diagnostic name is missing",
            )
        })?;
        self.write(
            self.call_register(operands.a)?,
            Value::register_function(target, name.to_owned(), captures, task_bound),
        )
    }

    pub(super) fn make_cell(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let value = self.read(self.call_register(operands.b)?)?.clone();
        self.write(
            self.call_register(operands.a)?,
            Value::Cell(Arc::new(Mutex::new(value))),
        )
    }

    pub(super) fn read_cell(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let cell_value = self.read(self.call_register(operands.b)?)?.clone();
        let Value::Cell(cell) = cell_value else {
            return Err(self.operand_type_error("cell", &cell_value));
        };
        let value = cell
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        self.write(self.call_register(operands.a)?, value)
    }

    pub(super) fn write_cell(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let cell_value = self.read(self.call_register(operands.a)?)?.clone();
        let value = self.read(self.call_register(operands.b)?)?.clone();
        let Value::Cell(cell) = cell_value else {
            return Err(self.operand_type_error("cell", &cell_value));
        };
        *cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = value;
        Ok(())
    }

    pub(super) fn return_from_call(&mut self, value: Value) -> Result<DispatchStep, VmError> {
        let Some(frame) = self.call_stack.pop() else {
            return Ok(DispatchStep::Return(value));
        };
        self.registers.truncate(self.base);
        self.function = frame.function;
        self.ip = frame.ip;
        self.base = frame.base;
        if let Some(destination) = frame.return_destination {
            let executable = self.executable.executable();
            let address = self.current_address;
            let slot = self.registers.get_mut(destination).ok_or_else(|| {
                diagnostics::internal(
                    executable,
                    address,
                    "Return destination left the caller frame",
                )
            })?;
            *slot = value;
        }
        Ok(DispatchStep::Continue)
    }

    fn enter_call(
        &mut self,
        target: FunctionId,
        destination: u16,
        arguments: Vec<Value>,
        captures: Vec<Value>,
    ) -> Result<(), VmError> {
        let image = self.executable.executable();
        let info = image
            .functions
            .get(usize::from(target.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    image,
                    self.current_address,
                    "Call target is outside the function table",
                )
            })?;
        if arguments.len() != usize::from(info.arity) {
            return Err(diagnostics::internal(
                image,
                self.current_address,
                format!(
                    "Call arity mismatch: expected {}, got {}",
                    info.arity,
                    arguments.len()
                ),
            ));
        }
        if captures.len() != usize::from(info.capture_count) {
            return Err(diagnostics::internal(
                image,
                self.current_address,
                format!(
                    "Closure capture mismatch: expected {}, got {}",
                    info.capture_count,
                    captures.len()
                ),
            ));
        }
        if self.call_stack.len() >= MAX_CALL_DEPTH {
            return Err(diagnostics::at_address(
                image,
                self.current_address,
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Call stack overflow",
                "Reduce recursion depth or replace recursion with iteration.",
            ));
        }
        let frame_size = usize::from(info.register_count);
        let new_len = self
            .registers
            .len()
            .checked_add(frame_size)
            .filter(|len| *len <= MAX_REGISTER_SLOTS)
            .ok_or_else(|| {
                diagnostics::at_address(
                    image,
                    self.current_address,
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Register stack overflow",
                    "Reduce recursion depth or the number of live registers per function.",
                )
            })?;
        let return_destination = if destination == fpas_bytecode::NO_REGISTER {
            None
        } else {
            Some(self.base + usize::from(destination))
        };
        let start = usize::try_from(info.code.start.get()).map_err(|_| {
            diagnostics::internal(
                image,
                self.current_address,
                "Callee address does not fit this host",
            )
        })?;
        self.call_stack.push(CallFrame {
            function: self.function,
            ip: self.ip,
            base: self.base,
            return_destination,
        });
        self.base = self.registers.len();
        self.registers.resize(new_len, Value::Unit);
        for (index, value) in arguments.into_iter().chain(captures).enumerate() {
            self.registers[self.base + index] = value;
        }
        self.function = target;
        self.ip = start;
        Ok(())
    }

    fn clone_window(&self, base: u16, count: u8) -> Result<Vec<Value>, VmError> {
        let start = self.base + usize::from(base);
        let end = start.checked_add(usize::from(count)).ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Register window overflow",
            )
        })?;
        self.registers
            .get(start..end)
            .map(<[Value]>::to_vec)
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Register window left the active frame",
                )
            })
    }

    fn operand_type_error(&self, expected: &str, actual: &Value) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected {expected}, got {}", actual.type_name()),
            format!("Use a {expected} value for this VM operation."),
        )
    }

    fn call_register(&self, value: u16) -> Result<Register, VmError> {
        Register::new(value).map_err(|error| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Verified register failed decoding: {error}"),
            )
        })
    }
}
