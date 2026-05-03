use super::super::Console;
use crate::error::{StdError, std_runtime_error};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{Command, QueueableCommand};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::io::Write;

impl Console {
    pub(super) fn enable_crt_mode(&mut self) {
        self.state.crt_mode = true;
    }

    pub fn enter_alt_screen(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.enable_crt_mode();
        self.run_writer_command(EnterAlternateScreen, "EnterAltScreen failed", location)
    }

    pub fn leave_alt_screen(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(LeaveAlternateScreen, "LeaveAltScreen failed", location)
    }

    pub fn enable_mouse(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(EnableMouseCapture, "EnableMouse failed", location)
    }

    pub fn disable_mouse(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(DisableMouseCapture, "DisableMouse failed", location)
    }

    pub fn enable_focus(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(EnableFocusChange, "EnableFocus failed", location)
    }

    pub fn disable_focus(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(DisableFocusChange, "DisableFocus failed", location)
    }

    pub fn enable_paste(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(EnableBracketedPaste, "EnablePaste failed", location)
    }

    pub fn disable_paste(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(DisableBracketedPaste, "DisablePaste failed", location)
    }

    pub(super) fn run_writer_command<C: Command>(
        &mut self,
        command: C,
        context: &str,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let Some(writer) = &mut self.writer else {
            return Ok(());
        };
        writer.queue(command).map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{context}: {e}"),
                "Run this in a terminal that supports screen control sequences.",
                location,
            )
        })?;
        writer.flush().map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{context} (flush): {e}"),
                "Check stdout availability and try again.",
                location,
            )
        })
    }
}
