//! Runtime state and operations for one hosted `Std.Tui` session.

mod input;
mod lifecycle;
mod redraw;

use super::damage::DamageTracker;
use crate::DamageRegion;
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

/// Runtime state for one hosted `Std.Tui` application session.
#[derive(Debug, Default)]
pub struct TuiSession {
    open: bool,
    damage: DamageTracker,
    redraw_hint: Option<DamageRegion>,
    owns_raw_mode: bool,
    owns_alt_screen: bool,
    owns_mouse: bool,
    /// When true, the session was opened with [`TuiSession::open_for_test`] (no terminal I/O).
    headless: bool,
}

impl TuiSession {
    fn ensure_open(
        &self,
        message: &'static str,
        help: &'static str,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.open {
            return Ok(());
        }

        Err(session_state_error(message, help, location))
    }
}

fn session_state_error(
    message: &'static str,
    help: &'static str,
    location: SourceLocation,
) -> StdError {
    std_runtime_error(RUNTIME_CONSOLE_STATE_ERROR, message, help, location)
}
