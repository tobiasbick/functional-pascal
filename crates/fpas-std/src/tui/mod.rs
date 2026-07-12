//! `Std.Tui` runtime: session, host bridge, commands, and dispatch helpers.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod cm_constants;
mod command;
mod event;
mod geometry;
mod host;
mod message_box_options;
mod session;

#[cfg(test)]
mod tests;

pub use cm_constants::{CM_ABOUT, CM_CANCEL, CM_CLOSE, CM_OK, CM_OPEN, CM_QUIT, CM_USER};
pub use command::{
    COMMAND_ID_CLOSE, COMMAND_ID_NEXT_WINDOW, COMMAND_ID_ZOOM, COMMAND_ID_ZOOM_BACK, CommandEvent,
    CommandId, CommandKind, CommandRegistry,
};
pub use event::TuiEvent;
pub use geometry::{ViewId, ViewRect};
pub use host::TuiHost;
pub use message_box_options::{
    MESSAGE_BOX_OPTION_ABOUT, MESSAGE_BOX_OPTION_CANCEL_BUTTON, MESSAGE_BOX_OPTION_CONFIRMATION,
    MESSAGE_BOX_OPTION_ERROR, MESSAGE_BOX_OPTION_INFORMATION, MESSAGE_BOX_OPTION_NO_BUTTON,
    MESSAGE_BOX_OPTION_OK_BUTTON, MESSAGE_BOX_OPTION_OK_CANCEL, MESSAGE_BOX_OPTION_WARNING,
    MESSAGE_BOX_OPTION_YES_BUTTON, MESSAGE_BOX_OPTION_YES_NO_CANCEL,
};
pub use session::TuiSession;
