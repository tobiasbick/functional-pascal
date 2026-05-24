//! `Std.Tui` shared semantic/compiler constants.
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

mod event;
mod session;

#[cfg(test)]
mod tests;

pub use event::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS, TuiEvent};
pub use session::TuiSession;
