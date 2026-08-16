//! Debuggee-channel facade: captured output stays session-owned; live input is rejected.

use super::*;
use crate::vm::debug::io::DebuggeeChannelState;

impl DebugSession {
    /// Return whether the session-owned debuggee channel is still open.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[must_use]
    pub const fn debuggee_channel_state(&self) -> DebuggeeChannelState {
        self.debuggee.state()
    }

    /// Reject live debuggee stdin without mutating workers, output, or the stop.
    ///
    /// Protocol stdin is JSONL or DAP. Queuing program input here would mix those
    /// bytes with hosted `Read`/`ReadLn` and is not a proven channel.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn push_debuggee_input(&mut self, _input: &str) -> DebugSessionError {
        if let Err(error) = self.require_stopped("io.input") {
            return error;
        }
        DebugSessionError {
            kind: DebugErrorKind::LiveInputUnsupported,
            message: "live debuggee input is not supported".to_string(),
            hint: "Keep JSONL or DAP bytes on the protocol stream. Program output uses structured `output` events; live stdin, TUI, and terminal input are not aliases of that stream.".to_string(),
        }
    }
}
