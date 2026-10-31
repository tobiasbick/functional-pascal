//! Repeated callback entry keeps immutable call metadata with its continuation.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`.

use super::{PreparedCall, Value, VmError, Worker, diagnostics};

impl Worker {
    /// Enter a one-argument callable without relying on a caller register window.
    /// The caller retains ownership of the operation and consumes its return separately.
    pub(in crate::execution::vm) fn enter_cleanup_callable(
        &mut self,
        function: &crate::SharedFunction,
        argument: Value,
    ) -> Result<(), VmError> {
        self.require_function_task_owner(function)?;
        let argument_count = 1 + usize::from(function.bound_receiver.is_some());
        let prepared = self.prepare_call::<false>(
            function.function,
            crate::NO_REGISTER,
            argument_count,
            function.captures.len(),
        )?;
        self.activate_call(&prepared, self.active_register_count())?;
        let mut index = self.base;
        if let Some(receiver) = &function.bound_receiver {
            self.store_register(index, receiver.clone())?;
            index += 1;
        }
        self.store_register(index, argument)?;
        index += 1;
        for capture in &function.captures {
            self.store_register(index, capture.clone())?;
            index += 1;
        }
        self.function = prepared.target;
        self.ip = prepared.instruction_pointer;
        Ok(())
    }

    /// Prepare only the first invocation; every invocation checks current stack limits.
    pub(in crate::execution::vm) fn enter_callback_inline(
        &mut self,
        arguments: impl ExactSizeIterator<Item = Value>,
    ) -> Result<PreparedCall, VmError> {
        let continuation = self.callback_continuations.last().ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Callback entry has no active continuation",
            )
        })?;
        let continuation_index = self.callback_continuations.len() - 1;
        let function = &continuation.callback;
        let argument_count = arguments
            .len()
            .saturating_add(usize::from(function.bound_receiver.is_some()));
        let prepared = if let Some(prepared) = continuation.prepared {
            self.require_call_depth()?;
            debug_assert_eq!(prepared.argument_count, argument_count);
            prepared
        } else {
            self.prepare_call::<false>(
                function.function,
                crate::NO_REGISTER,
                argument_count,
                function.captures.len(),
            )?
        };
        self.activate_call(&prepared, self.active_register_count())?;
        // Frame activation leaves the continuation in place. Its function can be borrowed
        // independently while argument and capture values enter the fresh register window.
        let Self {
            registers,
            register_initialized,
            callback_continuations,
            ..
        } = self;
        let function = &callback_continuations[continuation_index].callback;
        let base = self.base;
        let input_end = base + prepared.argument_count + function.captures.len();
        // Verified frame metadata bounds the complete input prefix. Remaining locals stay
        // uninitialized; execution access excludes maintenance during this frame setup.
        let (parameters, captures) =
            registers[base..input_end].split_at_mut(prepared.argument_count);
        let parameters = if let Some(receiver) = &function.bound_receiver {
            let (slot, parameters) = parameters.split_at_mut(1);
            slot[0] = receiver.clone();
            parameters
        } else {
            parameters
        };
        for (slot, value) in parameters.iter_mut().zip(arguments) {
            *slot = value;
        }
        for (slot, value) in captures.iter_mut().zip(&function.captures) {
            *slot = value.clone();
        }
        register_initialized[base..input_end].fill(true);
        self.function = prepared.target;
        self.ip = prepared.instruction_pointer;
        Ok(prepared)
    }
}
