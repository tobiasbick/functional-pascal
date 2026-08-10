//! Detached, effect-checked execution of debugger-side calls.

mod detach;
mod execute;

pub(super) use execute::CallSandbox;
