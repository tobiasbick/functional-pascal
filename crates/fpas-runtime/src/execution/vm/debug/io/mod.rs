//! Session-owned debuggee I/O distinct from JSONL and DAP protocol bytes.

mod channel;
mod terminal;

pub use crate::execution::vm::hosted::terminal_input::{
    DebugTerminalEvent, DebugTerminalKeyEvent, DebugTerminalKeyKind, DebugTerminalMouseAction,
    DebugTerminalMouseButton,
};
pub(in crate::execution::vm::debug) use channel::DebuggeeChannel;
pub use channel::DebuggeeChannelState;
pub use terminal::DebugTerminalHandle;
