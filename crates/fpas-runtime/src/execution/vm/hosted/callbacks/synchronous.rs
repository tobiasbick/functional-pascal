//! Immediate native and debugger execution using the rooted callback state machine.

use super::operation::{Advance, AdvanceError, CallbackContinuation};
use super::{Worker, plan};
use crate::execution::vm::VmError;
use crate::{Intrinsic, RootKind, SourceLocation, TraceRoots, Value};
use std::sync::{Arc, Mutex};

impl Worker {
    pub(in crate::execution::vm) fn execute_callback_intrinsic_sync(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        if let Some(value) = plan::inactive_result(self, intrinsic, arguments)? {
            return Ok(Some(Some(value)));
        }
        let Some((callback, operation)) = plan::operation(self, intrinsic, arguments, location)?
        else {
            return Ok(None);
        };
        let callback = self.resolve_callback(&callback, operation.first_arity())?;
        let state = Arc::new(NativeState(Mutex::new(CallbackContinuation::new(
            callback,
            None,
            operation,
            (self.function, self.current_address),
        ))));
        let _state_roots = self.domain.register_roots(RootKind::Native, state.clone());
        loop {
            let action = state
                .0
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .advance()
                .map_err(|error| match error {
                    AdvanceError::UnexpectedResult(value) => {
                        self.callback_type_error("boolean callback result", Some(&value))
                    }
                    AdvanceError::InvalidStringMapResult(value) => {
                        self.string_map_result_error(&value)
                    }
                    AdvanceError::InvalidState(message) => self.callback_state_error(message),
                    AdvanceError::Sorting(error) => error,
                })?;
            match action {
                Advance::Call(arguments) => {
                    let _arguments = self
                        .domain
                        .root_values(RootKind::Native, arguments.as_slice().to_vec());
                    let callback = state
                        .0
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .callback
                        .clone();
                    let result =
                        self.call_callback_sync(&Value::Function(callback), arguments.as_slice())?;
                    let mut state = state.0.lock().unwrap_or_else(|e| e.into_inner());
                    state.awaiting_depth = Some(0);
                    state
                        .accept(result)
                        .map_err(|message| self.callback_state_error(message))?;
                }
                Advance::Complete(value) => return Ok(Some(Some(value))),
                Advance::McpComplete(_, _) => {
                    return Err(self.callback_state_error("MCP callbacks require a serving task"));
                }
            }
        }
    }
}

struct NativeState(Mutex<CallbackContinuation>);

impl TraceRoots for NativeState {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), crate::OwnershipError> {
        let state = self.0.try_lock().map_err(|error| match error {
            std::sync::TryLockError::WouldBlock => crate::OwnershipError::Busy,
            std::sync::TryLockError::Poisoned(_) => crate::OwnershipError::Poisoned,
        })?;
        state.trace_roots(visit)
    }
}
