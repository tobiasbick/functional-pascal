//! Higher-order collection, result, and option intrinsics.

mod continuation;
mod operation;
mod plan;
mod synchronous;

pub(in crate::vm) use operation::CallbackContinuation;

use fpas_bytecode::{
    ArrayIntrinsic, DictIntrinsic, Intrinsic, OptionIntrinsic, ResultIntrinsic, SourceLocation,
    Value,
};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

use super::super::worker::Worker;
use super::super::{VmError, diagnostics};

/// Result of routing a higher-order hosted intrinsic.
pub(in crate::vm) enum CallbackOutcome {
    Complete(Value),
    Deferred,
}

/// Return whether an intrinsic requires callback execution owned by the VM.
pub(in crate::vm) fn is_callback_intrinsic(intrinsic: Intrinsic) -> bool {
    match intrinsic {
        Intrinsic::Array(operation) => matches!(
            operation,
            ArrayIntrinsic::Map
                | ArrayIntrinsic::Filter
                | ArrayIntrinsic::Reduce
                | ArrayIntrinsic::Find
                | ArrayIntrinsic::FindIndex
                | ArrayIntrinsic::Any
                | ArrayIntrinsic::All
                | ArrayIntrinsic::FlatMap
                | ArrayIntrinsic::ForEach
        ),
        Intrinsic::Dict(operation) => {
            matches!(operation, DictIntrinsic::Map | DictIntrinsic::Filter)
        }
        Intrinsic::Result(operation) => matches!(
            operation,
            ResultIntrinsic::Map | ResultIntrinsic::AndThen | ResultIntrinsic::OrElse
        ),
        Intrinsic::Option(operation) => matches!(
            operation,
            OptionIntrinsic::Map | OptionIntrinsic::AndThen | OptionIntrinsic::OrElse
        ),
        _ => false,
    }
}

impl Worker {
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
    pub(in crate::vm) fn accept_callback_return(&mut self, value: Value) {
        let continuation = self
            .callback_continuations
            .last_mut()
            .expect("callback return requires a continuation");
        continuation.accept(value);
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
}

#[cfg(test)]
mod tests {
    use fpas_bytecode::{
        ArrayIntrinsic, DictIntrinsic, Intrinsic, OptionIntrinsic, ResultIntrinsic,
    };

    use super::is_callback_intrinsic;

    #[test]
    fn classifier_selects_only_higher_order_intrinsics() {
        let callback_intrinsics = [
            Intrinsic::Array(ArrayIntrinsic::Map),
            Intrinsic::Array(ArrayIntrinsic::Filter),
            Intrinsic::Array(ArrayIntrinsic::Reduce),
            Intrinsic::Array(ArrayIntrinsic::Find),
            Intrinsic::Array(ArrayIntrinsic::FindIndex),
            Intrinsic::Array(ArrayIntrinsic::Any),
            Intrinsic::Array(ArrayIntrinsic::All),
            Intrinsic::Array(ArrayIntrinsic::FlatMap),
            Intrinsic::Array(ArrayIntrinsic::ForEach),
            Intrinsic::Dict(DictIntrinsic::Map),
            Intrinsic::Dict(DictIntrinsic::Filter),
            Intrinsic::Result(ResultIntrinsic::Map),
            Intrinsic::Result(ResultIntrinsic::AndThen),
            Intrinsic::Result(ResultIntrinsic::OrElse),
            Intrinsic::Option(OptionIntrinsic::Map),
            Intrinsic::Option(OptionIntrinsic::AndThen),
            Intrinsic::Option(OptionIntrinsic::OrElse),
        ];
        assert!(callback_intrinsics.into_iter().all(is_callback_intrinsic));

        let borrowed_intrinsics = [
            Intrinsic::Array(ArrayIntrinsic::Length),
            Intrinsic::Dict(DictIntrinsic::Length),
            Intrinsic::Result(ResultIntrinsic::Unwrap),
            Intrinsic::Option(OptionIntrinsic::Unwrap),
        ];
        assert!(
            borrowed_intrinsics
                .into_iter()
                .all(|intrinsic| !is_callback_intrinsic(intrinsic))
        );
    }
}
