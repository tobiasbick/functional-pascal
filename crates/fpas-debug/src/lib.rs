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

/// One-shot execution and immutable inspection for command-line agents.
pub mod batch;
pub mod dap;
pub mod jsonl;
/// Bounded serialization shared by batch and concurrent failure reports.
pub mod snapshot_value;

mod breakpoints;
mod engine;
mod evaluation;
mod target;
mod target_reload;
mod transport;

pub use target::{DebugSourceContent, PreparedDebugTarget, ReloadedDebugTarget};
