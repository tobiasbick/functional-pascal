//! Shared UI event types used internally across terminal and graphics hosts.
//!
//! Public Pascal-facing units still expose `Std.Console.Event`,
//! `Std.Tui.TuiEvent`, and `Std.Graph.Event` separately.

mod event;

pub use event::{UiEvent, UiModifiers, UiMouse, UiResize, UiWheel};
