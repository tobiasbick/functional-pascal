use super::super::Console;
use crate::console::screen::{
    TEXT_MODE_BW40, TEXT_MODE_BW80, TEXT_MODE_C40, TEXT_MODE_C80, TEXT_MODE_CO40, TEXT_MODE_CO80,
    TEXT_MODE_MONO, WindowRect,
};
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Console {
    /// Clears the active console window.
    pub fn clr_scr(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.clear_window();
        self.render_if_ready(location)
    }

    /// Clears from the cursor to the end of its line.
    pub fn clr_eol(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.clear_eol();
        self.render_if_ready(location)
    }

    /// `Std.Console.WindMin` as packed coordinate (low byte: X, high byte: Y).
    /// Spec: `docs/pascal/std/console/README.md`.
    pub fn wind_min(&self) -> i64 {
        self.state.wind_min()
    }

    /// `Std.Console.WindMax` as packed coordinate (low byte: X, high byte: Y).
    /// Spec: `docs/pascal/std/console/README.md`.
    pub fn wind_max(&self) -> i64 {
        self.state.wind_max()
    }

    /// Deletes the cursor row and shifts following rows upward.
    pub fn del_line(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.del_line();
        self.render_if_ready(location)
    }

    /// Inserts a blank cursor row and shifts following rows downward.
    pub fn ins_line(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.ins_line();
        self.render_if_ready(location)
    }

    /// Selects the inclusive one-based rectangle used by window-relative operations.
    pub fn window(
        &mut self,
        x1: i64,
        y1: i64,
        x2: i64,
        y2: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
            self.check_coord(x1, self.state.width),
            self.check_coord(y1, self.state.height),
            self.check_coord(x2, self.state.width),
            self.check_coord(y2, self.state.height),
        ) else {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Window({x1}, {y1}, {x2}, {y2}) is outside the current screen"),
                "Use inclusive coordinates from 1 through ScreenWidth and ScreenHeight.",
                location,
            ));
        };
        if x1 > x2 || y1 > y2 {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Window({x1}, {y1}, {x2}, {y2}) is invalid"),
                "Use inclusive screen coordinates where X1 <= X2 and Y1 <= Y2.",
                location,
            ));
        }

        self.state.set_window(WindowRect {
            left: x1,
            top: y1,
            right: x2,
            bottom: y2,
        });
        self.render_if_ready(location)
    }

    /// `Std.Console.TextMode(Mode)` — reset packed CRT state and clear the screen.
    pub fn text_mode(&mut self, mode: i64, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.validate_text_mode(mode, location)?;
        self.state.last_mode = mode;
        self.state
            .set_window(WindowRect::full(self.state.width, self.state.height));
        self.state.fg = 7;
        self.state.bg = 0;
        self.state.use_packed_colors();
        self.state.cursor_visible = true;
        self.state.cursor_big = false;
        self.state.clear_window();
        self.render_if_ready(location)
    }

    /// Returns the most recently selected CRT text mode.
    pub fn last_mode(&self) -> i64 {
        self.state.last_mode
    }

    /// Returns the current terminal width in cells.
    pub fn screen_width(&mut self) -> i64 {
        self.sync_terminal_size();
        self.state.screen_width()
    }

    /// Returns the current terminal height in cells.
    pub fn screen_height(&mut self) -> i64 {
        self.sync_terminal_size();
        self.state.screen_height()
    }

    fn validate_text_mode(&self, mode: i64, location: SourceLocation) -> Result<(), StdError> {
        if mode < 0 {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("TextMode expects a non-negative mode value, got {mode}"),
                "Use a CRT mode constant such as `C80`, `BW80`, or another non-negative integer.",
                location,
            ));
        }

        if matches!(
            mode,
            TEXT_MODE_BW40
                | TEXT_MODE_C40
                | TEXT_MODE_BW80
                | TEXT_MODE_C80
                | TEXT_MODE_CO40
                | TEXT_MODE_CO80
                | TEXT_MODE_MONO
                | 256
        ) {
            return Ok(());
        }

        Ok(())
    }
}
