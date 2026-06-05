//! Shared UI event types used internally across terminal and graphics hosts.
//!
//! Public Pascal-facing units still expose `Std.Console.Event`,
//! `Std.Tui.TuiEvent`, and `Std.Graph.Event` separately.

mod event;
mod host;

pub use event::{UiEvent, UiModifiers, UiMouse, UiResize, UiWheel};
pub use host::{UiHost, UiHostSurface};
