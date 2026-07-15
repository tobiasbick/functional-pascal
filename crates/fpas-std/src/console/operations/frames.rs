use super::super::Console;
use crate::error::StdError;
use fpas_bytecode::SourceLocation;

impl Console {
    /// Starts or nests a deferred console frame.
    ///
    /// **Documentation:** `docs/pascal/std/console/cells-frames.md`.
    pub fn begin_frame(&mut self) {
        if self.frame_depth == 0 {
            self.sync_terminal_size();
            self.enable_crt_mode();
        }
        self.frame_depth = self.frame_depth.saturating_add(1);
    }

    /// Completes one frame nesting level and flushes the outermost frame.
    ///
    /// Calling this outside a frame explicitly presents pending state.
    ///
    /// **Documentation:** `docs/pascal/std/console/cells-frames.md`.
    pub fn present(&mut self, location: SourceLocation) -> Result<(), StdError> {
        if self.frame_depth > 1 {
            self.frame_depth -= 1;
            return Ok(());
        }
        self.frame_depth = 0;
        self.enable_crt_mode();
        self.render_screen(location)
    }

    pub(super) fn render_if_ready(&mut self, location: SourceLocation) -> Result<(), StdError> {
        if self.frame_depth == 0 {
            self.render_screen(location)
        } else {
            Ok(())
        }
    }
}
