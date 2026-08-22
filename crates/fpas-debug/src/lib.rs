#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

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
