//! Validation for Pascal-supplied TUI view geometry.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md` (from the repository root).

use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::ViewRect;

pub(super) fn validate_view_rect(
    operation: &str,
    rect: ViewRect,
    line: SourceLocation,
) -> Result<ViewRect, VmError> {
    if !rect.is_empty() {
        return Ok(rect);
    }

    Err(runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "{operation} requires Width and Height greater than zero, got Width={} and Height={}",
            rect.width, rect.height
        ),
        "Pass positive terminal-cell dimensions; X and Y may be outside the visible screen.",
        line,
    ))
}
