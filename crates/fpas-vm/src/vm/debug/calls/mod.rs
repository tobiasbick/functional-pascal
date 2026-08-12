//! Detached, effect-checked execution of debugger-side calls.

mod detach;
mod enum_constructor;
mod execute;
mod resolution;

pub(super) use execute::CallSandbox;
