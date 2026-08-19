//! Protocol-neutral Functional Pascal source debugger frontends.

pub mod dap;
pub mod jsonl;

mod breakpoints;
mod evaluation;
mod target;
mod target_reload;
mod transport_input;

pub use target::{DebugSourceContent, PreparedDebugTarget, ReloadedDebugTarget};
