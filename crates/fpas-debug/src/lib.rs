//! Protocol-neutral Functional Pascal source debugger frontends.

pub mod dap;
pub mod jsonl;

mod breakpoints;
mod evaluation;
mod target;
mod target_reload;

pub use target::{DebugSourceContent, PreparedDebugTarget, ReloadedDebugTarget};
