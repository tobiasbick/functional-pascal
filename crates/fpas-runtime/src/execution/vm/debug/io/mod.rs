//! Session-owned debuggee I/O distinct from JSONL and DAP protocol bytes.

mod channel;
mod terminal;

pub(in crate::execution::vm::debug) use channel::DebuggeeChannel;
pub use channel::DebuggeeChannelState;
pub use terminal::{
    DebugTerminalEvent, DebugTerminalHandle, DebugTerminalKeyEvent, DebugTerminalKeyKind,
    DebugTerminalMouseAction, DebugTerminalMouseButton,
};
