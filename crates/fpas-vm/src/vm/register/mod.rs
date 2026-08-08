//! Safe register interpreter lifecycle through P5 globals and aggregates.

mod access;
mod callback;
mod calls;
mod diagnostics;
mod dispatch;
mod execute;
mod frame;
mod layouts;
mod worker;

#[cfg(test)]
mod tests;

use std::sync::{Arc, RwLock};

use fpas_bytecode::{FunctionId, Value, VerifiedExecutable};

use self::layouts::RuntimeLayouts;
use self::worker::RegisterWorker;
use super::VmError;

pub use callback::RegisterCallbackSession;

/// Successful root execution result and diagnostic instruction count.
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterExecution {
    /// Root or callback return value.
    pub value: Value,
    /// Number of packed instructions dispatched by this run.
    pub instruction_count: u64,
}

/// Single-use VM for a pre-verified register executable.
///
/// This API is intentionally not wired to the production CLI until later phases complete
/// intrinsics, tasks, persistent artifacts, and the production cutover.
pub struct RegisterVm {
    executable: Arc<VerifiedExecutable>,
    has_run: bool,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    layouts: Result<Arc<RuntimeLayouts>, VmError>,
}

impl RegisterVm {
    /// Construct an isolated VM from an owned verified executable.
    #[must_use]
    pub fn new(executable: VerifiedExecutable) -> Self {
        Self::from_shared(Arc::new(executable))
    }

    /// Construct an isolated VM sharing immutable verified executable metadata.
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
            has_run: false,
            globals,
            layouts,
        }
    }

    /// Execute the verified root function once.
    ///
    /// # Errors
    ///
    /// Returns the preserved runtime diagnostic for scalar failures, or an internal invariant
    /// diagnostic if a verified executable requests an opcode assigned to a later phase.
    pub fn run(&mut self) -> Result<RegisterExecution, VmError> {
        if std::mem::replace(&mut self.has_run, true) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                fpas_bytecode::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                "This register VM instance has already been run",
                "Register VM instances are single-use. Construct a new instance for each run.",
            ));
        }
        RegisterWorker::for_function_with_state(
            Arc::clone(&self.executable),
            self.executable.executable().entry,
            Vec::new(),
            Arc::clone(&self.globals),
            self.layouts.clone()?,
        )?
        .run()
    }

    /// Execute one numeric function as a synchronous hosted callback.
    ///
    /// # Errors
    ///
    /// Returns a runtime diagnostic for an invalid target, wrong arity, shutdown reuse, panic, or
    /// resource-limit failure. No function name lookup occurs.
    pub fn call(
        &mut self,
        function: FunctionId,
        arguments: Vec<Value>,
    ) -> Result<RegisterExecution, VmError> {
        if std::mem::replace(&mut self.has_run, true) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                fpas_bytecode::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                "This register VM instance has already been run",
                "Register VM instances are single-use. Construct a new instance for each callback.",
            ));
        }
        RegisterWorker::for_function_with_state(
            Arc::clone(&self.executable),
            function,
            arguments,
            Arc::clone(&self.globals),
            self.layouts.clone()?,
        )?
        .run()
    }
}
