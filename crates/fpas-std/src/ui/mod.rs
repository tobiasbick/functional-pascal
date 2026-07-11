//! Shared UI event types used internally across terminal and graphics hosts.
//!
//! Public Pascal-facing units expose `Std.Console.Event` and `Std.Graph.Event` separately.

mod event;
mod host;

pub use event::{UiEvent, UiModifiers, UiMouse, UiResize, UiWheel};
pub use host::{UiHost, UiHostSurface};
