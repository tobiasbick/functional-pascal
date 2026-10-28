//! Reusable synchronous callback execution for hosted intrinsics.

use std::sync::Arc;

use crate::{FunctionId, InstructionAddress, MemoryDomain, SharedFunction, Value};
use fpas_diagnostics::codes::{RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY};

use super::worker::{FunctionInputs, Worker};
use super::{VmError, diagnostics};

struct CallbackInputs<'a> {
    domain: MemoryDomain,
    bound_receiver: Option<&'a Value>,
    arguments: &'a [Value],
    captures: &'a [Value],
}

impl Worker {
    /// Invoke a first-class function synchronously through its numeric register target.
    pub(super) fn call_callback_sync(
        &self,
        callback: &Value,
        arguments: impl AsRef<[Value]>,
    ) -> Result<Value, VmError> {
        let arguments = arguments.as_ref();
        let function = self.validate_callback(callback, arguments.len())?;
        self.call_numeric_function(function, arguments)
    }

    /// Validate and clone a callback for a call with the given visible arity.
    pub(super) fn resolve_callback(
        &self,
        callback: &Value,
        argument_count: usize,
    ) -> Result<SharedFunction, VmError> {
        self.validate_callback(callback, argument_count).cloned()
    }

    /// Validate a callback without retaining its captured references.
    pub(super) fn validate_callback<'a>(
        &self,
        callback: &'a Value,
        argument_count: usize,
    ) -> Result<&'a SharedFunction, VmError> {
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
        if argument_count != visible_arity {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_WRONG_CALL_ARITY,
                format!(
                    "Function `{}` expects {} arguments, got {}",
                    function.name, visible_arity, argument_count
                ),
                "Check the callback signature and the intrinsic's callback contract.",
            ));
        }
        Ok(function)
    }

    fn call_numeric_function(
        &self,
        function: &SharedFunction,
        arguments: &[Value],
    ) -> Result<Value, VmError> {
        let _caller_roots = self.native_caller_roots()?;
        let _arguments = self.domain.root_values(
            crate::RootKind::Native,
            std::iter::once(Value::Function(function.clone()))
                .chain(arguments.iter().cloned())
                .collect(),
        );
        let target = function.function;
        let info = &self.executable.executable().functions[usize::from(target.get())];

        let mut callback = if let Some(mut callback) = self.callback_worker.borrow_mut().take() {
            callback.reset_for_callback(
                target,
                info.code.start,
                info.register_count,
                CallbackInputs {
                    domain: self.domain.clone(),
                    bound_receiver: function.bound_receiver.as_ref(),
                    arguments,
                    captures: &function.captures,
                },
                self.task_id,
            )?;
            callback
        } else {
            let call_arguments = function
                .bound_receiver
                .iter()
                .chain(arguments)
                .cloned()
                .collect::<Vec<_>>();
            let mut callback = Self::for_function_in_domain(
                Arc::clone(&self.executable),
                target,
                FunctionInputs {
                    arguments: &call_arguments,
                    captures: &function.captures,
                    domain: self.domain.clone(),
                },
                Arc::clone(&self.globals),
                Arc::clone(&self.layouts),
                Arc::clone(&self.hosted),
            )?
            .with_scheduler(self.scheduler.clone());
            callback.task_id = self.task_id;
            Box::new(callback)
        };
        callback.debug_trace = self.debug_trace.clone();
        callback.debug_local_trace = self.trace_callback_state();
        callback.debug_schedule = self.debug_schedule.clone();
        if self.debug_schedule.is_some() {
            callback.task_clock = self.task_clock.clone();
        }
        callback.inherit_scope_cancellation(self);
        callback.capture_record = self.capture_record.clone();
        callback.capture_callback = true;
        let execution = callback.run_in_place();
        if let Err(error) = &execution {
            self.capture_failure_continuation(error);
        }
        // Cached callback workers must not retain finished trace lifetimes or aliases.
        callback.debug_local_trace = None;
        callback.debug_trace = None;
        self.callback_failed.set(execution.is_err());
        if execution.is_err() {
            self.scope_failure_sequence
                .set(callback.scope_failure_sequence.take());
            self.observed_task_failure
                .set(callback.observed_task_failure.take());
        }
        if execution.is_ok() {
            callback.reset_registers(0);
            callback.call_stack.clear();
        }
        let _ = callback.publish_execution(&[]);
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
        inputs: CallbackInputs<'_>,
        task_id: u64,
    ) -> Result<(), VmError> {
        self.domain = inputs.domain;
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
        for (index, value) in inputs
            .bound_receiver
            .into_iter()
            .chain(inputs.arguments)
            .chain(inputs.captures)
            .enumerate()
        {
            self.store_register(index, value.clone())?;
        }
        self.call_stack.clear();
        self.instruction_count = 0;
        self.callback_instruction_count.set(0);
        self.callback_failed.set(false);
        self.current_address = start;
        self.observed_task_failure.set(None);
        self.task_id = task_id;
        self.retain_result = false;
        self.instructions_until_yield = super::TIMESLICE;
        self.suspend_requested = false;
        Ok(())
    }
}
