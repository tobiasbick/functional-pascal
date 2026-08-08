//! Synchronous hosted-callback entry using numeric function identifiers.

use std::sync::{Arc, RwLock};

use fpas_bytecode::{FunctionId, Value, VerifiedExecutable};
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

use super::hosted::HostedState;
use super::layouts::RuntimeLayouts;
use super::worker::Worker;
use super::{Execution, VmError, diagnostics};

/// Reusable synchronous callback runner for one verified executable.
///
/// Each callback receives fresh frame state while sharing immutable code and metadata. A callback
/// panic unwinds only that invocation; the session remains usable until cancelled or shut down.
pub struct CallbackSession {
    executable: Arc<VerifiedExecutable>,
    stopped: bool,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    layouts: Result<Arc<RuntimeLayouts>, VmError>,
    hosted: Arc<HostedState>,
}

impl CallbackSession {
    /// Create a callback session from an owned verified executable.
    #[must_use]
    pub fn new(executable: VerifiedExecutable) -> Self {
        Self::from_shared(Arc::new(executable))
    }

    /// Create a callback session sharing immutable executable storage.
    #[must_use]
    pub fn from_shared(executable: Arc<VerifiedExecutable>) -> Self {
        let globals = Arc::new(RwLock::new(vec![
            None;
            executable.executable().globals.len()
        ]));
        let layouts = RuntimeLayouts::build(
            executable.executable(),
            fpas_bytecode::InstructionAddress::new(0),
        )
        .map(Arc::new);
        Self {
            executable,
            stopped: false,
            globals,
            layouts,
            hosted: Arc::new(HostedState::new(fpas_std::Console::new(), Vec::new())),
        }
    }

    /// Invoke a numeric callback with arguments in source order.
    ///
    /// # Errors
    ///
    /// Returns callback diagnostics without poisoning the session. Calls after cancellation or
    /// shutdown return `RUNTIME_VM_SHUTDOWN` deterministically.
    pub fn invoke(
        &mut self,
        function: FunctionId,
        arguments: Vec<Value>,
    ) -> Result<Execution, VmError> {
        if self.stopped {
            return Err(stopped_error(self.executable.executable()));
        }
        Worker::for_function_with_state(
            Arc::clone(&self.executable),
            function,
            arguments,
            Arc::clone(&self.globals),
            self.layouts.clone()?,
            Arc::clone(&self.hosted),
        )?
        .run()
    }

    /// Cancel further hosted callback invocations.
    pub fn cancel(&mut self) {
        self.stopped = true;
    }

    /// Shut down the callback session.
    pub fn shutdown(&mut self) {
        self.stopped = true;
    }
}

fn stopped_error(executable: &fpas_bytecode::Executable) -> VmError {
    diagnostics::at_address(
        executable,
        fpas_bytecode::InstructionAddress::new(0),
        RUNTIME_VM_SHUTDOWN,
        "Register callback session is stopped",
        "Create a new callback session before invoking another callback.",
    )
}
