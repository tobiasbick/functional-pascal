//! Frame-root host bridge and chrome pointer dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

mod dispatch;
mod intrinsics;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation};

impl Worker {
    /// Execute frame-root host intrinsics.
    pub(super) fn try_exec_tui_frame_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        let Intrinsic::Tui(intrinsic) = intrinsic else {
            return Ok(false);
        };
        self.try_exec_tui_frame_tui_intrinsic(intrinsic, line)
    }
}
