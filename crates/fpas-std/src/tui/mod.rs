//! `Std.Tui` runtime: session, host bridge, views, widgets, and dispatch helpers.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

mod command;
mod damage;
mod event;
mod host;
mod modal;
mod session;
mod view;
mod widget;

#[cfg(test)]
mod tests;

pub use command::{CommandId, CommandRegistry};
pub use damage::DamageRegion;
pub use event::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS, TuiEvent};
pub use host::TuiHost;
pub use modal::{ModalId, ModalStack};
pub use session::TuiSession;
pub use view::{ViewId, ViewRect, ViewRegistry};
pub use widget::{
    MenuBarItem, MenuBarMouseResult, MenuBarStyle, MenuBarWidget, MenuPopupItem, SolidFillWidget,
    StatusBarSegment, StatusBarStyle, StatusBarWidget, ViewWidget,
};
