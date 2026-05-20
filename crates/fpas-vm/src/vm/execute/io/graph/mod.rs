//! `Std.Graph` VM execution and value/session bridging.
//!
//! **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

mod application;
mod records;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation};

impl Worker {
    /// Execute a `Std.Graph` intrinsic in the VM.
    pub(super) fn try_exec_graph_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        if self.try_exec_graph_application_intrinsic(intrinsic, line)? {
            return Ok(true);
        }

        Ok(false)
    }
}