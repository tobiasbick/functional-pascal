//! Integration tests for `Std.Tui` session lifecycle, redraw, and event handling.

#![allow(clippy::expect_used)]

mod events;
mod helpers;
mod lifecycle;
mod redraw;

pub(super) use super::{TuiEvent, TuiSession};
