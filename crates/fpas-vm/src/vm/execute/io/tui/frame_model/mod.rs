//! Frame-root host bridge and chrome pointer dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

mod dispatch;
mod intrinsics;

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::FrameGeometryError;

pub(super) fn frame_geometry_error(error: FrameGeometryError, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "frame geometry requires at least {}x{} cells, got {}x{}",
            error.min_width, error.min_height, error.got_width, error.got_height
        ),
        "Increase the requested width and height or disable scrollable frame chrome.",
        line,
    )
}

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
