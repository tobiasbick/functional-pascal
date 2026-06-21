//! `Std.Tui` runtime: session, host bridge, views, widgets, and dispatch helpers.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod command;
mod damage;
mod event;
mod host;
mod modal;
mod process;
mod scroll;
mod session;
mod view;
mod widget;

#[cfg(test)]
mod tests;

pub use command::{CommandEvent, CommandId, CommandKind, CommandRegistry};
pub use damage::DamageRegion;
pub use event::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS, TuiEvent};
pub use host::TuiHost;
pub use modal::{ModalClose, ModalId, ModalResult, ModalStack};
pub use process::{BlockedInput, FocusDirection, ProcessOutcome};
pub use scroll::{
    ScrollBarHit, ScrollBarOrientation, ScrollBarThumb, ScrollModel, drag_offset, hit_zone,
    thumb_geometry, track_cells,
};
pub use session::TuiSession;
pub use view::{
    DesktopMetrics, EventOutcome, EventPhase, EventRoute, ResolvedView, RootActivation,
    RoutedEvent, TUI_VIEW_KIND_VARIANTS, ViewId, ViewKind, ViewOptions, ViewRect, ViewRegistry,
    ViewState, WindowPalette, WindowShadow,
};
pub use widget::{
    ButtonStyle, ButtonWidget, CheckBoxStyle, CheckBoxWidget, FrameButtonSlots, FrameCapabilities,
    FrameContentSize, FrameGeometry, FrameGeometryError, FrameKind, FrameRoot, FrameRootSpec,
    FrameScrollbars, FramedDialogRoot, InputLineStyle, InputLineWidget, LabelStyle, LabelWidget,
    ListBoxItem, ListBoxStyle, ListBoxWidget, MenuBarItem, MenuBarMouseResult, MenuBarState,
    MenuBarStyle, MenuBarWidget, MenuPopupItem, RadioGroupStyle, RadioGroupWidget, RadioOption,
    ScrollBarStyle, ScrollBarWidget, ScrollViewStyle, ScrollViewWidget, SolidFillWidget,
    StatusBarSegment, StatusBarStyle, StatusBarWidget, ViewWidget, register_framed_dialog_root,
};
