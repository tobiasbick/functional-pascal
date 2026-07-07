//! `Std.Tui` runtime: session, host bridge, commands, and dispatch helpers.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod cm_constants;
mod command;
mod command_ids;
mod damage;
mod event;
mod geometry;
mod host;
mod message_box_options;
mod process;
mod session;

#[cfg(test)]
mod tests;

pub use command::{
    COMMAND_ID_CLOSE, COMMAND_ID_NEXT_WINDOW, COMMAND_ID_ZOOM, COMMAND_ID_ZOOM_BACK, CommandEvent,
    CommandId, CommandKind, CommandRegistry,
};
pub use command_ids::{COMMAND_CANCEL, COMMAND_CLOSE, COMMAND_OK, COMMAND_QUIT};
pub use damage::DamageRegion;
pub use event::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS, TuiEvent};
pub use geometry::{ViewId, ViewRect};
pub use host::TuiHost;
pub use message_box_options::{
    MESSAGE_BOX_OPTION_ABOUT, MESSAGE_BOX_OPTION_CANCEL_BUTTON, MESSAGE_BOX_OPTION_CONFIRMATION,
    MESSAGE_BOX_OPTION_ERROR, MESSAGE_BOX_OPTION_INFORMATION, MESSAGE_BOX_OPTION_NO_BUTTON,
    MESSAGE_BOX_OPTION_OK_BUTTON, MESSAGE_BOX_OPTION_OK_CANCEL, MESSAGE_BOX_OPTION_WARNING,
    MESSAGE_BOX_OPTION_YES_BUTTON, MESSAGE_BOX_OPTION_YES_NO_CANCEL,
};
pub use process::{BlockedInput, FocusDirection, ProcessOutcome};
pub use session::TuiSession;
