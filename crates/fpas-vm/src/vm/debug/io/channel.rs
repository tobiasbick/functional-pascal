//! Connect and close lifecycle for one debuggee I/O channel.

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
}

impl DebuggeeChannel {
    pub(in crate::vm::debug) const fn new() -> Self {
        Self {
            state: DebuggeeChannelState::Connected,
        }
    }

    pub(in crate::vm::debug) const fn state(self) -> DebuggeeChannelState {
        self.state
    }

    pub(in crate::vm::debug) fn close(&mut self) {
        self.state = DebuggeeChannelState::Closed;
    }
}
