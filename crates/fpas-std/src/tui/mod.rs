//! `Std.Tui` runtime: session, host bridge, commands, and dispatch helpers.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod command;
mod damage;
mod event;
mod geometry;
mod host;
mod process;
mod session;

#[cfg(test)]
mod tests;

pub use command::{
    COMMAND_ID_CLOSE, COMMAND_ID_NEXT_WINDOW, COMMAND_ID_ZOOM, COMMAND_ID_ZOOM_BACK, CommandEvent,
    CommandId, CommandKind, CommandRegistry,
};
pub use damage::DamageRegion;
pub use event::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS, TuiEvent};
pub use geometry::{ViewId, ViewRect};
pub use host::TuiHost;
pub use process::{BlockedInput, FocusDirection, ProcessOutcome};
pub use session::TuiSession;
