//! Debuggee-channel facade: queued line input, EOF, cancel, and disconnect cleanup.

use super::*;
use crate::vm::debug::io::DebuggeeChannelState;
use crate::vm::debug::types::DebuggeeInputResult;
#[cfg(test)]
use fpas_std::KeyInput;
use fpas_std::TextInput;

impl DebugSession {
    /// Return whether the session-owned debuggee channel is still open.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[must_use]
    pub const fn debuggee_channel_state(&self) -> DebuggeeChannelState {
        self.debuggee.state()
    }

    /// Queue one line for hosted `Read` / `ReadLn` without touching protocol stdin.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn push_debuggee_input(
        &mut self,
        input: &str,
    ) -> Result<DebuggeeInputResult, DebugSessionError> {
        self.require_stopped("io.input")?;
        let bytes = input.len().saturating_add(1);
        match self.debuggee.accept_line(bytes) {
            Ok(session_bytes) => {
                self.with_text_input(|text| text.push_line(input));
                Ok(DebuggeeInputResult {
                    bytes,
                    session_bytes,
                })
            }
            Err(kind) => Err(debuggee_input_error(kind)),
        }
    }

    /// Signal debuggee input EOF without dispatching bytecode.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn signal_debuggee_eof(&mut self) -> Result<(), DebugSessionError> {
        self.require_stopped("io.eof")?;
        self.debuggee.signal_eof().map_err(debuggee_input_error)?;
        self.with_text_input(TextInput::close_input);
        Ok(())
    }

    /// Drop unread queued debuggee input without closing the channel.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn cancel_debuggee_input(&mut self) -> Result<(), DebugSessionError> {
        self.require_stopped("io.cancel")?;
        if self.debuggee.state() != DebuggeeChannelState::Connected {
            return Err(debuggee_input_error(DebugErrorKind::InvalidState));
        }
        self.with_text_input(TextInput::clear_queued);
        Ok(())
    }

    /// Apply `operation` to the hosted text-input queue.
    pub(super) fn with_text_input<R>(&self, operation: impl FnOnce(&mut TextInput) -> R) -> R {
        let Some(worker) = self.runtime.worker(0) else {
            unreachable!("debug runtime always retains the main task")
        };
        let mut input = worker
            .hosted
            .text_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut input)
    }

    /// Apply `operation` to the hosted keyboard and TUI event queue.
    #[cfg(test)]
    pub(super) fn with_key_input<R>(&self, operation: impl FnOnce(&mut KeyInput) -> R) -> R {
        let Some(worker) = self.runtime.worker(0) else {
            unreachable!("debug runtime always retains the main task")
        };
        let mut input = worker
            .hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut input)
    }
}

fn debuggee_input_error(kind: DebugErrorKind) -> DebugSessionError {
    match kind {
        DebugErrorKind::DebuggeeInputLimit => DebugSessionError {
            kind,
            message: "debuggee input exceeds the session input limit".to_string(),
            hint: "Send fewer lines, or raise the debugger input limit.".to_string(),
        },
        DebugErrorKind::DebuggeeInputClosed => DebugSessionError {
            kind,
            message: "debuggee input is closed".to_string(),
            hint: "Do not queue more lines after EOF or disconnect.".to_string(),
        },
        _ => DebugSessionError {
            kind: DebugErrorKind::InvalidState,
            message: "debuggee input is unavailable".to_string(),
            hint: "Use io.input only while the debug session is stopped and connected.".to_string(),
        },
    }
}
