//! Reusable synchronous callback execution for hosted intrinsics.

use std::sync::Arc;

use fpas_bytecode::{FunctionId, InstructionAddress, SharedFunction, Value};
use fpas_diagnostics::codes::{RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY};

use super::worker::Worker;
use super::{VmError, diagnostics};

impl Worker {
    /// Invoke a first-class function synchronously through its numeric register target.
    pub(super) fn call_callback_sync(
        &self,
        callback: &Value,
        arguments: impl AsRef<[Value]>,
    ) -> Result<Value, VmError> {
        let Value::Function(function) = callback else {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected function value, got `{}`", callback.type_name()),
                "Pass a named function or function-typed variable as the callback argument.",
            ));
        };
        self.call_numeric_function(function, arguments.as_ref())
    }

    fn call_numeric_function(
        &self,
        function: &SharedFunction,
        arguments: &[Value],
    ) -> Result<Value, VmError> {
        let target = function.function;
        let info = self
            .executable
            .executable()
            .functions
            .get(usize::from(target.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Callback target is outside the function table",
                )
            })?;
        if arguments.len() != usize::from(info.arity) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_WRONG_CALL_ARITY,
                format!(
                    "Function `{}` expects {} arguments, got {}",
                    function.name,
                    info.arity,
                    arguments.len()
                ),
                "Check the callback signature and the intrinsic's callback contract.",
            ));
        }

        let mut callback = if let Some(mut callback) = self.callback_worker.borrow_mut().take() {
            callback.reset_for_callback(
                target,
                info.code.start,
                info.register_count,
                arguments,
                &function.captures,
            )?;
            callback
        } else {
            Box::new(
                Self::for_function_with_captures(
                    Arc::clone(&self.executable),
                    target,
                    arguments,
                    &function.captures,
                    Arc::clone(&self.globals),
                    Arc::clone(&self.layouts),
                    Arc::clone(&self.hosted),
                )?
                .with_scheduler(self.scheduler.clone()),
            )
        };
        let execution = callback.run_in_place();
        *self.callback_worker.borrow_mut() = Some(callback);
        let execution = execution?;
        self.callback_instruction_count.set(
            self.callback_instruction_count
                .get()
                .saturating_add(execution.instruction_count),
        );
        Ok(execution.value)
    }

    fn reset_for_callback(
        &mut self,
        target: FunctionId,
        start: InstructionAddress,
        register_count: u16,
        arguments: &[Value],
        captures: &[Value],
    ) -> Result<(), VmError> {
        self.function = target;
        self.ip = usize::try_from(start.get()).map_err(|_| {
            diagnostics::internal(
                self.executable.executable(),
                start,
                "Callback address does not fit this host",
            )
        })?;
        self.base = 0;
        self.reset_registers(usize::from(register_count));
        for (index, value) in arguments.iter().chain(captures).enumerate() {
            self.store_register(index, value.clone())?;
        }
        self.call_stack.clear();
        self.instruction_count = 0;
        self.callback_instruction_count.set(0);
        self.current_address = start;
        self.task_id = 0;
        self.retain_result = false;
        self.instructions_until_yield = super::TIMESLICE;
        self.suspend_requested = false;
        Ok(())
    }
}
