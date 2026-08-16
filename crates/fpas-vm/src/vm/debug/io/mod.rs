//! Session-owned debuggee I/O distinct from JSONL and DAP protocol bytes.

mod channel;

pub(in crate::vm::debug) use channel::DebuggeeChannel;
pub use channel::DebuggeeChannelState;
