//! Higher-order collection, result, and option intrinsics.

mod arguments;
mod continuation;
mod operation;
mod plan;
mod string;
mod synchronous;

pub(in crate::vm) use operation::CallbackContinuation;

use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

use super::super::worker::Worker;
use super::super::{VmError, diagnostics};

/// Result of routing a higher-order hosted intrinsic.
pub(in crate::vm) enum CallbackOutcome {
    Complete(Value),
    Deferred,
}

impl Worker {
    /// Invoke the winning selection callback through the resumable task-owned path.
    pub(in crate::vm) fn start_selection_callback(
        &mut self,
        callback: fpas_bytecode::SharedFunction,
        argument: Option<Value>,
        index: usize,
        destination: Option<fpas_bytecode::Register>,
    ) -> Result<(), VmError> {
        let arguments = argument.map_or_else(
            arguments::CallbackArguments::empty,
            arguments::CallbackArguments::one,
        );
        self.callback_continuations.push(CallbackContinuation::new(
            callback,
            destination.map(|register| self.base + usize::from(register.get())),
            operation::CallbackOperation::Single {
                arguments: Some(arguments),
                wrapper: operation::SingleWrapper::SelectionIndex(index as i64),
            },
        ));
        continuation::resume(self)?;
        Ok(())
    }
    /// Execute immediately on the main task or retain resumable state for a spawned task.
    pub(in crate::vm) fn execute_callback_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        _location: SourceLocation,
        destination: Option<usize>,
    ) -> Result<Option<CallbackOutcome>, VmError> {
        if self.task_id == 0 {
            return self
                .execute_callback_intrinsic_sync(intrinsic, arguments, _location)
                .map(|result| result.flatten().map(CallbackOutcome::Complete));
        }
        continuation::start(self, intrinsic, arguments, destination)
    }

    /// Advance the active hosted operation when its callback is not running.
    pub(in crate::vm) fn resume_callback_continuation(&mut self) -> Result<bool, VmError> {
        continuation::resume(self)
    }

    /// Return whether the current function return completes the active hosted callback.
    pub(in crate::vm) fn callback_accepts_return(&self) -> bool {
        self.callback_continuations
            .last()
            .is_some_and(|continuation| continuation.awaits_depth(self.call_stack.len()))
    }

    /// Save one callback result for the next hosted-operation step.
    pub(in crate::vm) fn accept_callback_return(&mut self, value: Value) -> Result<(), VmError> {
        let Some(continuation) = self.callback_continuations.last_mut() else {
            return Err(self.callback_state_error("Callback return has no active continuation"));
        };
        continuation
            .accept(value)
            .map_err(|message| self.callback_state_error(message))
    }

    fn array_argument<'a>(
        &self,
        value: Option<&'a Value>,
        _context: &str,
    ) -> Result<&'a [Value], VmError> {
        match value {
            Some(Value::Array(values)) => Ok(values),
            other => Err(self.callback_type_error("array", other)),
        }
    }

    fn arity_error(&self, context: &str) -> VmError {
        diagnostics::internal(
            self.executable.executable(),
            self.current_address,
            format!("Verified {context} arguments are incomplete"),
        )
    }

    fn callback_type_error(&self, expected: &str, actual: Option<&Value>) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Expected {expected}, got {}",
                actual.map_or("missing argument", Value::type_name)
            ),
            format!("Pass a {expected} value to this intrinsic."),
        )
    }

    fn string_map_result_error(&self, actual: &Value) -> VmError {
        let detail = match actual {
            Value::Str(value) => format!("{} Unicode scalars", value.char_len()),
            value => format!("a `{}` value", value.type_name()),
        };
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Std.Str.Map callback must return exactly one Unicode scalar, got {detail}"),
            "Return a string containing exactly one Unicode scalar from the callback.",
        )
    }

    fn callback_state_error(&self, message: &str) -> VmError {
        diagnostics::internal(self.executable.executable(), self.current_address, message)
    }
}
