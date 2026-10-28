//! Worker-owned orchestration for inline higher-order callbacks.

use crate::{Intrinsic, SourceLocation, Value};

use super::operation::{Advance, AdvanceError, CallbackContinuation};
use super::{CallbackOutcome, Worker, plan};
use crate::execution::vm::VmError;

/// Start a supported callback operation on its main or scheduled owner.
pub(super) fn start(
    worker: &mut Worker,
    intrinsic: Intrinsic,
    arguments: &[Value],
    location: SourceLocation,
    destination: Option<usize>,
) -> Result<Option<CallbackOutcome>, VmError> {
    if let Some(value) = plan::inactive_result(worker, intrinsic, arguments)? {
        return Ok(Some(CallbackOutcome::Complete(value)));
    }
    let Some((callback, operation)) = plan::operation(worker, intrinsic, arguments, location)?
    else {
        return Ok(None);
    };
    let callback = worker.resolve_callback(&callback, operation.first_arity())?;
    worker
        .callback_continuations
        .push(CallbackContinuation::new(
            callback,
            destination,
            operation,
            (worker.function, worker.current_address),
        ));
    resume(worker)?;
    Ok(Some(CallbackOutcome::Deferred))
}

/// Resume the top hosted callback operation if its callback has returned.
pub(super) fn resume(worker: &mut Worker) -> Result<bool, VmError> {
    let action = {
        let Some(continuation) = worker.callback_continuations.last_mut() else {
            return Ok(false);
        };
        if continuation.awaiting_depth.is_some() {
            return Ok(false);
        }
        continuation.advance()
    };
    let action = action.map_err(|error| match error {
        AdvanceError::UnexpectedResult(value) => {
            worker.callback_type_error("boolean callback result", Some(&value))
        }
        AdvanceError::InvalidStringMapResult(value) => worker.string_map_result_error(&value),
        AdvanceError::InvalidState(message) => worker.callback_state_error(message),
        AdvanceError::Sorting(error) => error,
    })?;
    match action {
        Advance::McpComplete(invocation, value) => {
            worker.callback_continuations.pop();
            worker.finish_mcp_callback(*invocation, value)?;
        }
        Advance::Call(arguments) => {
            let prepared = worker.enter_callback_inline(arguments.into_values())?;
            let awaiting_depth = worker.call_stack.len();
            let Some(continuation) = worker.callback_continuations.last_mut() else {
                return Err(worker.callback_state_error(
                    "Callback continuation disappeared while entering its call",
                ));
            };
            continuation.awaiting_depth = Some(awaiting_depth);
            continuation.prepared = Some(prepared);
        }
        Advance::Complete(value) => {
            let Some(continuation) = worker.callback_continuations.pop() else {
                return Err(worker.callback_state_error(
                    "Completed callback operation has no active continuation",
                ));
            };
            if let Some(destination) = continuation.destination {
                worker.store_register(destination, value)?;
            }
        }
    }
    Ok(true)
}
