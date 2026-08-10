//! Protocol-neutral Functional Pascal source debugger frontends.

pub mod dap;
pub mod jsonl;

mod target;

pub use target::{DebugSourceContent, PreparedDebugTarget};
