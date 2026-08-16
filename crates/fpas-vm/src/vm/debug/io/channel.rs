//! Connect, queue, EOF, and close lifecycle for one debuggee I/O channel.

use super::super::types::DebugErrorKind;

/// Whether the session still owns an open debuggee channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggeeChannelState {
    /// Launch has opened the channel; captured output may accumulate.
    Connected,
    /// Disconnect closed the channel without further program I/O.
    Closed,
}

/// Session-owned debuggee channel that must never share unframed protocol bytes.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebuggeeChannel {
    state: DebuggeeChannelState,
    eof: bool,
    accepted_bytes: usize,
    max_bytes: usize,
}

impl DebuggeeChannel {
    pub(in crate::vm::debug) const fn new(max_bytes: usize) -> Self {
        Self {
            state: DebuggeeChannelState::Connected,
            eof: false,
            accepted_bytes: 0,
            max_bytes,
        }
    }

    pub(in crate::vm::debug) const fn state(self) -> DebuggeeChannelState {
        self.state
    }

    pub(in crate::vm::debug) fn close(&mut self) {
        self.state = DebuggeeChannelState::Closed;
        self.eof = true;
    }

    pub(in crate::vm::debug) fn signal_eof(&mut self) -> Result<(), DebugErrorKind> {
        if self.state != DebuggeeChannelState::Connected {
            return Err(DebugErrorKind::InvalidState);
        }
        self.eof = true;
        Ok(())
    }

    pub(in crate::vm::debug) fn accept_line(
        &mut self,
        bytes: usize,
    ) -> Result<usize, DebugErrorKind> {
        if self.state != DebuggeeChannelState::Connected {
            return Err(DebugErrorKind::InvalidState);
        }
        if self.eof {
            return Err(DebugErrorKind::DebuggeeInputClosed);
        }
        let session_bytes = self.accepted_bytes.saturating_add(bytes);
        if session_bytes > self.max_bytes {
            return Err(DebugErrorKind::DebuggeeInputLimit);
        }
        self.accepted_bytes = session_bytes;
        Ok(session_bytes)
    }
}
