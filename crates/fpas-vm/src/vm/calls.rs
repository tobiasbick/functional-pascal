//! Numeric calls and frame transitions.

mod closures;
mod tail_call;

use std::sync::{Arc, Mutex};

use fpas_bytecode::{AbcOperands, FunctionId, Register, SharedFunction, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::dispatch::DispatchStep;
use super::frame::{CallFrame, MAX_CALL_DEPTH, MAX_REGISTER_SLOTS};
use super::worker::Worker;
use super::{VmError, diagnostics};

struct PreparedCall {
    target: FunctionId,
    frame_size: usize,
    return_destination: Option<usize>,
    instruction_pointer: usize,
    argument_count: usize,
}

impl Worker {
    pub(super) fn call_direct(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let target = FunctionId::new(operands.b);
        self.enter_call(target, operands.a, operands.c, operands.auxiliary, &[], &[])
    }

    pub(super) fn call_value(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let callee = self.read(self.call_register(operands.b)?)?.clone();
        let Value::Function(function) = callee else {
            return Err(self.operand_type_error("function", &callee));
        };
        self.require_function_task_owner(&function)?;
        let target = function.function;
        let receiver = function.bound_receiver.as_ref().map(std::slice::from_ref);
        self.enter_call(
            target,
            operands.a,
            operands.c,
            operands.auxiliary,
            receiver.unwrap_or_default(),
            &function.captures,
        )
    }

    pub(super) fn require_function_task_owner(
        &self,
        function: &SharedFunction,
    ) -> Result<(), VmError> {
        if !function.task_bound || function.owner_task == Some(self.task_id) {
            return Ok(());
        }
        Err(diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_INVALID_TASK,
            format!(
                "Cannot invoke task-bound closure `{}` from a foreign task",
                function.name
            ),
            "Invoke the closure on the task that owns it. Mutable captures keep a function on one task.",
        ))
    }

    pub(super) fn return_from_call(&mut self, value: Value) -> Result<DispatchStep, VmError> {
        let callback_return = self.callback_accepts_return();
        let Some(frame) = self.call_stack.pop() else {
            return Ok(DispatchStep::Return(value));
        };
        self.release_registers(self.base);
        self.restore_caller_window(&frame);
        self.function = frame.function;
        self.ip = frame.ip;
        self.base = frame.base;
        if callback_return {
            self.accept_callback_return(value)?;
        } else if let Some(destination) = frame.return_destination {
            self.store_register(destination, value)?;
        }
        Ok(DispatchStep::Continue)
    }

    fn enter_call(
        &mut self,
        target: FunctionId,
        destination: u16,
        argument_base: u16,
        argument_count: u8,
        prefix_arguments: &[Value],
        captures: &[Value],
    ) -> Result<(), VmError> {
        let argument_start = self
            .base
            .checked_add(usize::from(argument_base))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Call argument window overflowed the active frame",
                )
            })?;
        let argument_end = argument_start
            .checked_add(usize::from(argument_count))
            .filter(|end| *end <= self.active_register_count())
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Call argument window left the active frame",
                )
            })?;
        let actual_argument_count =
            usize::from(argument_count).saturating_add(prefix_arguments.len());
        let prepared =
            self.prepare_call(target, destination, actual_argument_count, captures.len())?;
        // Arguments that end the caller's frame already sit where the callee's parameters go:
        // the callee frame starts on them instead of copying (overlapping register windows).
        let overlapping = prefix_arguments.is_empty()
            && argument_count > 0
            && argument_end == self.active_register_count();
        let callee_base = if overlapping {
            argument_start
        } else {
            self.active_register_count()
        };
        self.activate_call(&prepared, callee_base)?;
        if !overlapping {
            for (index, value) in prefix_arguments.iter().enumerate() {
                self.store_register(self.base + index, value.clone())?;
            }
            for (index, source) in (argument_start..argument_end).enumerate() {
                self.store_register(
                    self.base + prefix_arguments.len() + index,
                    self.registers[source].clone(),
                )?;
            }
        }
        for (index, value) in captures.iter().enumerate() {
            self.store_register(self.base + prepared.argument_count + index, value.clone())?;
        }
        self.function = prepared.target;
        self.ip = prepared.instruction_pointer;
        Ok(())
    }

    /// Enter a hosted callback on this worker so its task can suspend and resume normally.
    pub(super) fn enter_callback_inline(
        &mut self,
        function: &SharedFunction,
        arguments: &[Value],
    ) -> Result<(), VmError> {
        let argument_count = arguments
            .len()
            .saturating_add(usize::from(function.bound_receiver.is_some()));
        let prepared = self.prepare_call(
            function.function,
            fpas_bytecode::NO_REGISTER,
            argument_count,
            function.captures.len(),
        )?;
        self.activate_call(&prepared, self.active_register_count())?;
        let mut next = 0;
        if let Some(receiver) = &function.bound_receiver {
            self.store_register(self.base, receiver.clone())?;
            next = 1;
        }
        for (index, value) in arguments.iter().enumerate() {
            self.store_register(self.base + next + index, value.clone())?;
        }
        for (index, value) in function.captures.iter().enumerate() {
            self.store_register(self.base + prepared.argument_count + index, value.clone())?;
        }
        self.function = prepared.target;
        self.ip = prepared.instruction_pointer;
        Ok(())
    }

    fn prepare_call(
        &self,
        target: FunctionId,
        destination: u16,
        argument_count: usize,
        capture_count: usize,
    ) -> Result<PreparedCall, VmError> {
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
        if argument_count != usize::from(info.arity) {
            return Err(diagnostics::internal(
                image,
                self.current_address,
                format!(
                    "Call arity mismatch: expected {}, got {}",
                    info.arity, argument_count
                ),
            ));
        }
        if capture_count != usize::from(info.capture_count) {
            return Err(diagnostics::internal(
                image,
                self.current_address,
                format!(
                    "Closure capture mismatch: expected {}, got {}",
                    info.capture_count, capture_count
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
        let return_destination = if destination == fpas_bytecode::NO_REGISTER {
            None
        } else {
            Some(self.base + usize::from(destination))
        };
        let instruction_pointer = usize::try_from(info.code.start.get()).map_err(|_| {
            diagnostics::internal(
                image,
                self.current_address,
                "Callee address does not fit this host",
            )
        })?;
        Ok(PreparedCall {
            target,
            frame_size,
            return_destination,
            instruction_pointer,
            argument_count,
        })
    }

    /// Push the caller frame and activate the callee frame starting at `callee_base`.
    fn activate_call(
        &mut self,
        prepared: &PreparedCall,
        callee_base: usize,
    ) -> Result<(), VmError> {
        let frame_end = callee_base
            .checked_add(prepared.frame_size)
            .filter(|end| *end <= MAX_REGISTER_SLOTS)
            .ok_or_else(|| {
                diagnostics::at_address(
                    self.executable.executable(),
                    self.current_address,
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Register stack overflow",
                    "Reduce recursion depth or the number of live registers per function.",
                )
            })?;
        self.call_stack.push(CallFrame {
            function: self.function,
            ip: self.ip,
            base: self.base,
            return_destination: prepared.return_destination,
            frame_end: self.active_register_count(),
        });
        self.base = callee_base;
        self.activate_registers(frame_end);
        Ok(())
    }

    /// Re-activate caller registers that an overlapping callee frame released on return.
    pub(super) fn restore_caller_window(&mut self, frame: &CallFrame) {
        if self.active_register_count() < frame.frame_end {
            self.activate_registers(frame.frame_end);
        }
    }

    pub(super) fn clone_window(&self, base: u16, count: u8) -> Result<Vec<Value>, VmError> {
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
