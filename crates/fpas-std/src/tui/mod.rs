//! `Std.Tui` runtime: session, host bridge, views, widgets, and dispatch helpers.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod command;
mod damage;
mod event;
mod host;
mod modal;
mod process;
mod session;
mod view;
mod widget;

#[cfg(test)]
mod tests;

pub use command::{CommandEvent, CommandId, CommandKind, CommandRegistry};
pub use damage::DamageRegion;
pub use event::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS, TuiEvent};
pub use host::TuiHost;
pub use modal::{ModalId, ModalStack};
pub use process::{BlockedInput, FocusDirection, ProcessOutcome};
pub use session::TuiSession;
pub use view::{
    DesktopMetrics, EventOutcome, EventPhase, EventRoute, ResolvedView, RootActivation,
    RoutedEvent, ViewId, ViewOptions, ViewRect, ViewRegistry, ViewState, WindowPalette,
    WindowShadow,
};
pub use widget::{
    MenuBarItem, MenuBarMouseResult, MenuBarState, MenuBarStyle, MenuBarWidget, MenuPopupItem,
    SolidFillWidget, StatusBarSegment, StatusBarStyle, StatusBarWidget, ViewWidget,
};
