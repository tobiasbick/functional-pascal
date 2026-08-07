//! Safe P3 register interpreter lifecycle.

mod access;
mod diagnostics;
mod dispatch;
mod execute;
mod worker;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use fpas_bytecode::{Value, VerifiedExecutable};

use self::worker::RegisterWorker;
use super::VmError;

/// Successful root execution result and diagnostic instruction count.
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterExecution {
    /// Root return value; P3 root programs always return Unit.
    pub value: Value,
    /// Number of packed instructions dispatched by this run.
    pub instruction_count: u64,
}

/// Single-use VM for a pre-verified register executable.
///
/// This API is intentionally not wired to the production CLI until later cutover phases complete
/// calls, aggregates, intrinsics, tasks, and persistent artifacts.
pub struct RegisterVm {
    executable: Arc<VerifiedExecutable>,
    has_run: bool,
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
        Self {
            executable,
            has_run: false,
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
        RegisterWorker::new(Arc::clone(&self.executable))?.run()
    }
}
