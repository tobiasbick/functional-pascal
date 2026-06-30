//! Turbo Vision geometry and handle error helpers shared across bridge modules.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::TurboVisionRect;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

pub(super) fn state_rect(rect: turbo_vision::core::geometry::Rect) -> TurboVisionRect {
    TurboVisionRect {
        x: rect.a.x,
        y: rect.a.y,
        width: rect.b.x - rect.a.x,
        height: rect.b.y - rect.a.y,
    }
}

pub(super) fn turbo_rect(rect: TurboVisionRect) -> turbo_vision::core::geometry::Rect {
    turbo_vision::core::geometry::Rect::from_coords(rect.x, rect.y, rect.width, rect.height)
}

pub(super) fn unknown_handle_error(
    label: &'static str,
    handle: u32,
    line: SourceLocation,
) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("{label} handle {handle} is not live"),
        "Use a handle returned by the matching Turbo Vision constructor in the active application session.",
        line,
    )
}
