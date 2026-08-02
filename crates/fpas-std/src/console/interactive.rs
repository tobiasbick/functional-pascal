//! Transactional interactive terminal session ownership for fullscreen Console use.
//!
//! **Documentation:** `docs/pascal/std/console/events.md`.

use super::{Console, KeyInput};
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::io::IsTerminal;

/// Modes successfully taken by [`Console::acquire_interactive_terminal`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct InteractiveTerminalOwnership {
    /// True while an interactive session is open (even when no writer is attached).
    pub acquired: bool,
    pub owns_alt_screen: bool,
    pub owns_mouse: bool,
    pub owns_focus: bool,
    pub owns_paste: bool,
    pub owns_cursor_hidden: bool,
}

impl Console {
    /// Acquire exclusive interactive terminal ownership.
    ///
    /// When a writer is attached, enables raw mode, alternate screen, mouse, focus, paste, and
    /// hides the cursor. Steps roll back in reverse on failure. Without a writer the call only
    /// records ownership so a second acquire fails.
    pub fn acquire_interactive_terminal(
        &mut self,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.interactive.acquired {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "AcquireInteractiveTerminal cannot open a second interactive terminal session.",
                "Call `ReleaseInteractiveTerminal()` before acquiring again.",
                location,
            ));
        }

        self.interactive.acquired = true;
        if !self.has_terminal_writer() {
            return Ok(());
        }

        // Shared-buffer and piped CI writers are not interactive TTYs; skip raw mode there so
        // acquire remains usable in tests while still owning screen/mouse/focus/paste/cursor.
        if std::io::stdin().is_terminal()
            && let Err(error) = key_input.enable_raw_mode_explicit(location)
        {
            self.interactive.acquired = false;
            return Err(error);
        }

        if let Err(error) = self.enter_alt_screen(location) {
            self.interactive.acquired = key_input.disable_raw_mode_explicit(location).is_err();
            return Err(error);
        }
        self.interactive.owns_alt_screen = true;

        if let Err(error) = self.enable_mouse(location) {
            self.rollback_partial_acquire(key_input, location);
            return Err(error);
        }
        self.interactive.owns_mouse = true;

        if let Err(error) = self.enable_focus(location) {
            self.rollback_partial_acquire(key_input, location);
            return Err(error);
        }
        self.interactive.owns_focus = true;

        if let Err(error) = self.enable_paste(location) {
            self.rollback_partial_acquire(key_input, location);
            return Err(error);
        }
        self.interactive.owns_paste = true;

        if let Err(error) = self.cursor_off(location) {
            self.rollback_partial_acquire(key_input, location);
            return Err(error);
        }
        self.interactive.owns_cursor_hidden = true;
        Ok(())
    }

    /// Restore modes owned by the interactive session. Idempotent when nothing is acquired.
    pub fn release_interactive_terminal(
        &mut self,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if !self.interactive.acquired {
            return Ok(());
        }

        let mut first_error = self.restore_owned_console_modes(location);
        if let Err(error) = key_input.disable_raw_mode_explicit(location) {
            record_first_error(&mut first_error, error);
        }
        if let Some(error) = first_error {
            return Err(error);
        }

        self.clear_interactive_ownership();
        Ok(())
    }

    /// Returns whether an interactive terminal session is currently acquired.
    #[must_use]
    pub fn interactive_terminal_acquired(&self) -> bool {
        self.interactive.acquired
    }

    /// Restore console-owned interactive modes without requiring [`KeyInput`].
    ///
    /// Raw mode remains the responsibility of [`KeyInput`]'s Drop / explicit disable.
    pub(crate) fn restore_interactive_console_modes(&mut self) {
        if !self.interactive.acquired {
            return;
        }
        let location = SourceLocation::new(1, 1);
        if self.restore_owned_console_modes(location).is_none() {
            self.clear_interactive_ownership();
        }
    }

    fn rollback_partial_acquire(&mut self, key_input: &mut KeyInput, location: SourceLocation) {
        let console_error = self.restore_owned_console_modes(location);
        let raw_mode_restored = key_input.disable_raw_mode_explicit(location).is_ok();
        if console_error.is_none() && raw_mode_restored {
            self.clear_interactive_ownership();
        }
    }

    fn restore_owned_console_modes(&mut self, location: SourceLocation) -> Option<StdError> {
        let mut ownership = std::mem::take(&mut self.interactive);
        let mut first_error = None;

        if ownership.owns_cursor_hidden {
            let result = self.cursor_on(location);
            record_mode_restoration(&mut ownership.owns_cursor_hidden, result, &mut first_error);
        }
        if ownership.owns_paste {
            let result = self.disable_paste(location);
            record_mode_restoration(&mut ownership.owns_paste, result, &mut first_error);
        }
        if ownership.owns_focus {
            let result = self.disable_focus(location);
            record_mode_restoration(&mut ownership.owns_focus, result, &mut first_error);
        }
        if ownership.owns_mouse {
            let result = self.disable_mouse(location);
            record_mode_restoration(&mut ownership.owns_mouse, result, &mut first_error);
        }
        if ownership.owns_alt_screen {
            let result = self.leave_alt_screen(location);
            record_mode_restoration(&mut ownership.owns_alt_screen, result, &mut first_error);
        }

        self.interactive = ownership;
        first_error
    }

    fn clear_interactive_ownership(&mut self) {
        self.interactive = InteractiveTerminalOwnership::default();
    }
}

fn record_mode_restoration(
    owned: &mut bool,
    result: Result<(), StdError>,
    first_error: &mut Option<StdError>,
) {
    match result {
        Ok(()) => *owned = false,
        Err(error) => record_first_error(first_error, error),
    }
}

fn record_first_error(first_error: &mut Option<StdError>, error: StdError) {
    if first_error.is_none() {
        *first_error = Some(error);
    }
}
