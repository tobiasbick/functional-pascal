//! `Std.Graph` VM execution and value/session bridging.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md`, `docs/pascal/std/graph/app.md` (from the repository root).

mod application;
mod handlers;
mod host;
mod records;
mod test_host;

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
        if self.try_exec_graph_application_intrinsic(intrinsic, line)?
            || self.try_exec_graph_host_intrinsic(intrinsic, line)?
            || self.try_exec_graph_test_host_intrinsic(intrinsic, line)?
        {
            return Ok(true);
        }

        Ok(false)
    }
}
