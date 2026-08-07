//! Synchronous hosted-callback entry using numeric function identifiers.

use std::sync::Arc;

use fpas_bytecode::{FunctionId, Value, VerifiedExecutable};
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

use super::worker::RegisterWorker;
use super::{RegisterExecution, VmError, diagnostics};

/// Reusable synchronous callback runner for one verified executable.
///
/// Each callback receives fresh frame state while sharing immutable code and metadata. A callback
/// panic unwinds only that invocation; the session remains usable until cancelled or shut down.
pub struct RegisterCallbackSession {
    executable: Arc<VerifiedExecutable>,
    stopped: bool,
}

impl RegisterCallbackSession {
    /// Create a callback session from an owned verified executable.
    #[must_use]
    pub fn new(executable: VerifiedExecutable) -> Self {
        Self::from_shared(Arc::new(executable))
    }

    /// Create a callback session sharing immutable executable storage.
    #[must_use]
    pub fn from_shared(executable: Arc<VerifiedExecutable>) -> Self {
        Self {
            executable,
            stopped: false,
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
    ) -> Result<RegisterExecution, VmError> {
        if self.stopped {
            return Err(stopped_error(self.executable.executable()));
        }
        RegisterWorker::for_function(Arc::clone(&self.executable), function, arguments)?.run()
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
