//! Higher-order collection, result, and option intrinsics.

mod arguments;
mod continuation;
mod operation;
mod plan;
mod sorting;
mod string;
mod synchronous;

pub(in crate::execution::vm) use operation::CallbackContinuation;

use crate::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

use super::super::worker::Worker;
use super::super::{VmError, diagnostics};

/// Result of routing a higher-order hosted intrinsic.
pub(in crate::execution::vm) enum CallbackOutcome {
    Complete(Value),
    Deferred,
}

impl Worker {
    pub(in crate::execution::vm) fn start_mcp_callback(
        &mut self,
        callback: crate::SharedFunction,
        context: Value,
        argument: Value,
        invocation: Box<crate::execution::vm::hosted::mcp::wait::Invocation>,
    ) -> Result<(), VmError> {
        let origin = invocation.origin();
        self.callback_continuations.push(CallbackContinuation::new(
            callback,
            None,
            operation::CallbackOperation::Mcp {
                arguments: Some(arguments::CallbackArguments::two(context, argument)),
                invocation: Some(invocation),
            },
            origin,
        ));
        continuation::resume(self)?;
        Ok(())
    }
    /// Invoke the winning selection callback through the resumable task-owned path.
    pub(in crate::execution::vm) fn start_selection_callback(
        &mut self,
        callback: crate::SharedFunction,
        argument: Option<Value>,
        index: usize,
        destination: Option<crate::Register>,
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
            (self.function, self.current_address),
        ));
        continuation::resume(self)?;
        Ok(())
    }
    /// Execute higher-order bytecode calls through the owning worker's frames.
    pub(in crate::execution::vm) fn execute_callback_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
        destination: Option<usize>,
    ) -> Result<Option<CallbackOutcome>, VmError> {
        continuation::start(self, intrinsic, arguments, location, destination)
    }

    /// Advance the active hosted operation when its callback is not running.
    pub(in crate::execution::vm) fn resume_callback_continuation(
        &mut self,
    ) -> Result<bool, VmError> {
        continuation::resume(self)
    }

    /// Original instruction owning the top resumable operation and its result store.
    pub(in crate::execution::vm) fn callback_origin(
        &self,
    ) -> Option<(crate::FunctionId, crate::InstructionAddress)> {
        self.callback_continuations.last().map(|state| state.origin)
    }

    /// Return whether the top hosted callback operation is ready for its next step.
    pub(in crate::execution::vm) fn callback_resume_pending(&self) -> bool {
        self.callback_continuations
            .last()
            .is_some_and(|continuation| continuation.awaiting_depth.is_none())
    }

    /// Return whether the current function return completes the active hosted callback.
    pub(in crate::execution::vm) fn callback_accepts_return(&self) -> bool {
        self.callback_continuations
            .last()
            .is_some_and(|continuation| continuation.awaits_depth(self.call_stack.len()))
    }

    /// Save one callback result for the next hosted-operation step.
    pub(in crate::execution::vm) fn accept_callback_return(
        &mut self,
        value: Value,
    ) -> Result<(), VmError> {
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
    ) -> Result<&'a crate::SharedArray, VmError> {
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
