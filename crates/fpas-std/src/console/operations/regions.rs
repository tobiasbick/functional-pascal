use super::super::{Console, ConsoleRect, SavedRegionId};
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Console {
    /// Saves a clipped screen region and returns an opaque one-shot handle.
    pub fn save_region(
        &mut self,
        rect: ConsoleRect,
        location: SourceLocation,
    ) -> Result<SavedRegionId, StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.save_region(rect).ok_or_else(|| {
            std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "SaveRegion received an empty or out-of-bounds rectangle",
                "Use positive 1-based coordinates and a non-empty rectangle within the screen.",
                location,
            )
        })
    }

    /// Restores and consumes a saved region.
    pub fn restore_region(
        &mut self,
        id: SavedRegionId,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if !self.state.restore_region(id) {
            return Err(Self::invalid_saved_region("RestoreRegion", location));
        }
        self.render_if_ready(location)
    }

    /// Releases a saved region without restoring it.
    pub fn discard_region(
        &mut self,
        id: SavedRegionId,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if !self.state.discard_region(id) {
            return Err(Self::invalid_saved_region("DiscardRegion", location));
        }
        Ok(())
    }

    fn invalid_saved_region(operation: &str, location: SourceLocation) -> StdError {
        std_runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            format!("{operation} received an expired or unknown SavedRegion"),
            "Use each SavedRegion once: restore it or discard it, but not both.",
            location,
        )
    }
}
