//! `Std.Tui` VM execution — intrinsic dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod application;
mod damage;
mod handlers;
mod host;
mod modal_host;
mod query_host;
mod records;
mod test_host;
mod turbo_vision;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation};

impl Worker {
    /// Execute a `Std.Tui` intrinsic in the VM.
    pub(super) fn try_exec_tui_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        if self.try_exec_tui_application_intrinsic(intrinsic, line)?
            || self.try_exec_turbo_vision_intrinsic(intrinsic, line)?
            || self.try_exec_tui_test_host_intrinsic(intrinsic, line)?
            || self.try_exec_tui_query_host_intrinsic(intrinsic, line)?
            || self.try_exec_tui_host_intrinsic(intrinsic, line)?
        {
            return Ok(true);
        }

        Ok(false)
    }
}
