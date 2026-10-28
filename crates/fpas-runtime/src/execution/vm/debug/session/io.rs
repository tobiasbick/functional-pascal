//! Debuggee-channel facade: queued line input, EOF, cancel, and disconnect cleanup.

use super::*;
use crate::execution::vm::debug::io::DebuggeeChannelState;
use crate::execution::vm::debug::types::DebuggeeInputResult;
use crate::stdlib::KeyInput;
use crate::stdlib::TextInput;

impl DebugSession {
    /// Supply separate child-process streams without exposing debugger protocol stdin.
    /// Seeded scheduling still rejects external process effects.
    pub fn set_interactive_process_terminal(
        &mut self,
        terminal: crate::execution::InteractiveProcessTerminal,
    ) -> Result<(), DebugSessionError> {
        self.require_stopped("process.terminal")?;
        let Some(worker) = self.runtime.worker(0) else {
            return Err(DebugSessionError::invalid_state(
                "process.terminal",
                self.state,
            ));
        };
        *worker
            .hosted
            .process_terminal
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(terminal);
        Ok(())
    }

    /// Return whether the session-owned debuggee channel is still open.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[must_use]
    pub fn debuggee_channel_state(&self) -> DebuggeeChannelState {
        self.debuggee
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .state()
    }

    /// Queue one line for hosted `ReadText` / `ReadLn` without touching protocol stdin.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn push_debuggee_input(
        &mut self,
        input: &str,
    ) -> Result<DebuggeeInputResult, DebugSessionError> {
        self.require_stopped("io.input")?;
        let bytes = input.len().saturating_add(1);
        let accepted = self
            .debuggee
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .accept_line(bytes);
        match accepted {
            Ok(session_bytes) => {
                self.with_text_input(|text| text.push_line(input));
                self.capture_input(input);
                Ok(DebuggeeInputResult {
                    bytes,
                    session_bytes,
                })
            }
            Err(kind) => Err(debuggee_input_error(kind)),
        }
    }

    /// Queue runner-compatible characters without host terminal reads or bytecode execution.
    pub fn push_debuggee_keys(
        &mut self,
        input: &str,
    ) -> Result<DebuggeeInputResult, DebugSessionError> {
        self.require_stopped("io.keys")?;
        let bytes = input.len();
        let session_bytes = self
            .debuggee
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .accept_input(bytes)
            .map_err(debuggee_input_error)?;
        self.with_key_input(|keys| keys.push_chars(input));
        Ok(DebuggeeInputResult {
            bytes,
            session_bytes,
        })
    }

    /// Signal debuggee input EOF without dispatching bytecode.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn signal_debuggee_eof(&mut self) -> Result<(), DebugSessionError> {
        self.require_stopped("io.eof")?;
        self.debuggee
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .signal_eof()
            .map_err(debuggee_input_error)?;
        self.with_text_input(TextInput::close_input);
        Ok(())
    }

    /// Drop unread queued debuggee input without closing the channel.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn cancel_debuggee_input(&mut self) -> Result<(), DebugSessionError> {
        self.require_stopped("io.cancel")?;
        if self
            .debuggee
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .state()
            != DebuggeeChannelState::Connected
        {
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
