//! Task-owned orchestration for resumable hosted callbacks.

use fpas_bytecode::{Intrinsic, Value};

use super::operation::{Advance, CallbackContinuation};
use super::{CallbackOutcome, Worker, plan};
use crate::vm::VmError;

/// Start a supported hosted callback operation for a spawned task.
pub(super) fn start(
    worker: &mut Worker,
    intrinsic: Intrinsic,
    arguments: &[Value],
    destination: Option<usize>,
) -> Result<Option<CallbackOutcome>, VmError> {
    if let Some(value) = plan::inactive_result(worker, intrinsic, arguments)? {
        return Ok(Some(CallbackOutcome::Complete(value)));
    }
    let Some((callback, operation)) = plan::operation(worker, intrinsic, arguments)? else {
        return Ok(None);
    };
    let callback = worker.resolve_callback(&callback, operation.first_arity())?;
    worker
        .callback_continuations
        .push(CallbackContinuation::new(callback, destination, operation));
    resume(worker)?;
    Ok(Some(CallbackOutcome::Deferred))
}

/// Resume the top hosted callback operation if its callback has returned.
pub(super) fn resume(worker: &mut Worker) -> Result<bool, VmError> {
    let (action, callback) = {
        let Some(continuation) = worker.callback_continuations.last_mut() else {
            return Ok(false);
        };
        if continuation.awaiting_depth.is_some() {
            return Ok(false);
        }
        (continuation.advance(), continuation.callback.clone())
    };
    let action = action
        .map_err(|value| worker.callback_type_error("boolean callback result", Some(&value)))?;
    match action {
        Advance::Call(arguments) => {
            worker.enter_callback_inline(&callback, &arguments)?;
            worker
                .callback_continuations
                .last_mut()
                .expect("callback continuation remains active")
                .awaiting_depth = Some(worker.call_stack.len());
        }
        Advance::Complete(value) => {
            let continuation = worker
                .callback_continuations
                .pop()
                .expect("completed callback continuation exists");
            if let Some(destination) = continuation.destination {
                worker.store_register(destination, value)?;
            }
        }
    }
    Ok(true)
}
