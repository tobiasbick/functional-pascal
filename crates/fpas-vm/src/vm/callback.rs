//! Synchronous hosted-callback entry using numeric function identifiers.

#[cfg(test)]
mod domains;
#[cfg(test)]
mod fs_lifecycle;

use std::sync::{Arc, RwLock};

use fpas_bytecode::{FunctionId, MemoryDomain, Value, VerifiedExecutable};
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

use super::host_value::HostValues;
use super::hosted::HostedState;
use super::layouts::RuntimeLayouts;
use super::worker::{FunctionInputs, Worker};
use super::{Execution, HostArgument, VmError, diagnostics};

/// Reusable synchronous callback runner for one verified executable.
///
/// Each callback receives fresh frame state while sharing immutable code and metadata. A callback
/// panic unwinds only that invocation; the session remains usable until cancelled or shut down.
pub struct CallbackSession {
    domain: MemoryDomain,
    host_values: HostValues,
    executable: Arc<VerifiedExecutable>,
    stopped: bool,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    layouts: Result<Arc<RuntimeLayouts>, VmError>,
    hosted: Arc<HostedState>,
    session: Option<fpas_bytecode::DomainSession>,
}

impl CallbackSession {
    /// Configure collection for this reusable session and its nested execution.
    #[must_use]
    pub fn with_collection_schedule(self, schedule: fpas_bytecode::CollectionSchedule) -> Self {
        crate::vm::domain::configure(&self.domain, schedule);
        self
    }

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
        let (domain, session) = crate::vm::domain::new_session_domain();
        Self {
            host_values: HostValues::for_executable(
                domain.clone(),
                &executable,
                Arc::clone(&globals),
            ),
            domain,
            executable,
            stopped: false,
            session: Some(session),
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
        arguments: Vec<HostArgument>,
    ) -> Result<Execution, VmError> {
        if self.stopped {
            return Err(stopped_error(self.executable.executable()));
        }
        let layouts = self.layouts.clone()?;
        let (arguments, argument_roots) = self.host_values.admit(arguments, &self.executable)?;
        let worker = Worker::for_function_in_domain(
            Arc::clone(&self.executable),
            function,
            FunctionInputs {
                arguments: &arguments,
                captures: &[],
                domain: self.domain.clone(),
            },
            Arc::clone(&self.globals),
            layouts,
            Arc::clone(&self.hosted),
        );
        fpas_bytecode::release_values(arguments);
        drop(argument_roots);
        worker?
            .run()
            .map(|execution| self.host_values.export(execution))
    }

    /// Cancel further hosted callback invocations.
    pub fn cancel(&mut self) {
        self.stopped = true;
        self.hosted.close_process_handles();
        self.hosted.close_file_handles();
        self.hosted.wait_process_handles();
        self.hosted.cases.close_all();
        self.session.take();
    }

    /// Shut down the callback session.
    pub fn shutdown(&mut self) {
        self.cancel();
    }
}

impl Drop for CallbackSession {
    fn drop(&mut self) {
        self.hosted.close_process_handles();
        self.hosted.close_file_handles();
        self.hosted.wait_process_handles();
        self.hosted.cases.close_all();
        self.host_values.retire_globals(&self.globals);
        self.session.take();
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
