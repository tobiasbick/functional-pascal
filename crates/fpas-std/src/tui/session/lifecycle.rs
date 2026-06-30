//! Session setup, teardown, mode ownership, and size queries.

use super::{TuiSession, session_state_error};
use crate::console::{Console, KeyInput};
use crate::error::StdError;
use fpas_bytecode::SourceLocation;

impl TuiSession {
    /// Open a TUI application session and acquire terminal state when a writer is available.
    pub fn open(
        &mut self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.open_deferred(console, location)?;
        self.acquire_terminal(console, key_input, location)
    }

    /// Open a TUI application session without acquiring terminal state.
    ///
    /// This is used when the concrete backend owns the terminal lifecycle itself.
    pub fn open_deferred(
        &mut self,
        console: &mut Console,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.open {
            return Err(session_state_error(
                "Application.Open() cannot open a second Std.Tui session while one is already active.",
                "Close the current application with `Application.Close(App)` before opening a new one.",
                location,
            ));
        }

        self.open = true;
        self.headless = false;
        self.damage.clear();
        self.redraw_hint = None;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;
        self.owns_mouse = false;
        console.abort_tui_paint();

        Ok(())
    }

    /// Acquire terminal state for the retained terminal backend if it is not owned already.
    pub fn acquire_terminal(
        &mut self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Application.Run(App) requires an open Std.Tui application session.",
            "Open the application first and keep the returned handle alive while running it.",
            location,
        )?;
        if self.headless || self.owns_raw_mode || self.owns_alt_screen || self.owns_mouse {
            return Ok(());
        }
        if !console.has_terminal_writer() {
            return Ok(());
        }

        if let Err(error) = key_input.enable_raw_mode_explicit(location) {
            self.open = false;
            return Err(error);
        }
        self.owns_raw_mode = true;

        if let Err(error) = console.enter_alt_screen(location) {
            let _ = key_input.disable_raw_mode_explicit(location);
            self.open = false;
            self.owns_raw_mode = false;
            return Err(error);
        }

        self.owns_alt_screen = true;

        if let Err(error) = console.enable_mouse(location) {
            let _ = console.leave_alt_screen(location);
            let _ = key_input.disable_raw_mode_explicit(location);
            self.open = false;
            self.owns_raw_mode = false;
            self.owns_alt_screen = false;
            return Err(error);
        }
        self.owns_mouse = true;
        Ok(())
    }

    /// Close the active TUI application session and restore terminal state.
    pub fn close(
        &mut self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Application.Close() requires an open Std.Tui application session.",
            "Call `Application.Open()` before closing the application session.",
            location,
        )?;

        let mut first_error = None;
        console.abort_tui_paint();

        if self.owns_alt_screen
            && let Err(error) = console.leave_alt_screen(location)
        {
            first_error = Some(error);
        }

        if self.owns_raw_mode
            && let Err(error) = key_input.disable_raw_mode_explicit(location)
            && first_error.is_none()
        {
            first_error = Some(error);
        }

        if self.owns_mouse
            && let Err(error) = console.disable_mouse(location)
            && first_error.is_none()
        {
            first_error = Some(error);
        }

        self.open = false;
        self.damage.clear();
        self.redraw_hint = None;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;
        self.owns_mouse = false;
        self.headless = false;

        if let Some(error) = first_error {
            return Err(error);
        }

        Ok(())
    }

    /// Open a headless TUI session for native FPAS tests (`Application.OpenForTest`).
    ///
    /// Does not acquire raw mode, alternate screen, or mouse capture. Resize the logical
    /// console to the desired virtual terminal size before calling this method.
    pub fn open_for_test(
        &mut self,
        console: &mut Console,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.open {
            return Err(session_state_error(
                "Application.OpenForTest() cannot open a second Std.Tui session while one is already active.",
                "Close the current application with `Application.CloseForTest(App)` before opening a new one.",
                location,
            ));
        }

        console.abort_tui_paint();
        self.open = true;
        self.headless = true;
        self.damage.clear();
        self.redraw_hint = None;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;
        self.owns_mouse = false;
        Ok(())
    }

    /// Returns whether this session was opened headlessly for native tests.
    #[must_use]
    pub fn is_headless(&self) -> bool {
        self.headless
    }

    /// Return the current terminal size for the active application session.
    pub fn size(
        &self,
        console: &mut Console,
        location: SourceLocation,
    ) -> Result<(i64, i64), StdError> {
        self.ensure_open(
            "Application.Size(App) requires an open Std.Tui application session.",
            "Open the application first and keep the returned handle alive while querying its size.",
            location,
        )?;

        Ok((console.screen_width(), console.screen_height()))
    }
}
