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
        self.require_function_task_owner(function)?;
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
        let visible_arity = usize::from(info.arity)
            .checked_sub(usize::from(function.bound_receiver.is_some()))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Bound callback target has no receiver parameter",
                )
            })?;
        if arguments.len() != visible_arity {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_WRONG_CALL_ARITY,
                format!(
                    "Function `{}` expects {} arguments, got {}",
                    function.name,
                    visible_arity,
                    arguments.len()
                ),
                "Check the callback signature and the intrinsic's callback contract.",
            ));
        }

        let call_arguments = function
            .bound_receiver
            .iter()
            .chain(arguments)
            .cloned()
            .collect::<Vec<_>>();
        let mut callback = if let Some(mut callback) = self.callback_worker.borrow_mut().take() {
            callback.reset_for_callback(
                target,
                info.code.start,
                info.register_count,
                &call_arguments,
                &function.captures,
                self.task_id,
            )?;
            callback
        } else {
            let mut callback = Self::for_function_with_captures(
                Arc::clone(&self.executable),
                target,
                &call_arguments,
                &function.captures,
                Arc::clone(&self.globals),
                Arc::clone(&self.layouts),
                Arc::clone(&self.hosted),
            )?
            .with_scheduler(self.scheduler.clone());
            callback.task_id = self.task_id;
            Box::new(callback)
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
        task_id: u64,
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
        self.task_id = task_id;
        self.retain_result = false;
        self.instructions_until_yield = super::TIMESLICE;
        self.suspend_requested = false;
        Ok(())
    }
}
