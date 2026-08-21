//! Protocol-neutral Functional Pascal source debugger frontends.

pub mod dap;
pub mod jsonl;

mod breakpoints;
mod engine;
mod evaluation;
mod target;
mod target_reload;
mod transport;

pub use target::{DebugSourceContent, PreparedDebugTarget, ReloadedDebugTarget};
