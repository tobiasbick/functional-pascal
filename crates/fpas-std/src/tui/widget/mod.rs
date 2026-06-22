//! Rust-hosted TUI widgets painted directly by the VM host.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui/app/README.md`

mod control;
pub(crate) mod frame;
mod menu_bar;
mod menu_label_paint;
mod menu_popup;
mod menu_style;
mod solid_fill;
mod status_bar;

pub use control::{
    ButtonStyle, ButtonWidget, CheckBoxStyle, CheckBoxWidget, InputLineStyle, InputLineWidget,
    LabelStyle, LabelWidget, ListBoxItem, ListBoxStyle, ListBoxWidget, RadioGroupStyle,
    RadioGroupWidget, RadioOption, ScrollBarStyle, ScrollBarWidget, ScrollViewStyle,
    ScrollViewWidget,
};
pub use frame::{
    FrameButtonSlots, FrameCapabilities, FrameChromeHit, FrameContentSize, FrameGeometry,
    FrameGeometryError, FrameKind, FrameRoot, FrameRootSpec, FrameRootState, FrameScrollbars,
    FrameStyle, FrameWidget, FramedDialogRoot, register_framed_dialog_root,
};
pub use menu_bar::{MenuBarItem, MenuBarMouseResult, MenuBarState, MenuBarStyle, MenuBarWidget};
pub use menu_popup::MenuPopupItem;
pub use solid_fill::SolidFillWidget;
pub use status_bar::{StatusBarSegment, StatusBarStyle, StatusBarWidget};

use crate::{Console, DamageRegion, ViewKind, ViewRect, ViewState};

/// Native widget attached to a host-managed view.
///
/// Spec: `docs/pascal/std/tui/app/README.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewWidget {
    /// Solid CRT-color fill, optionally tiled with one character.
    SolidFill(SolidFillWidget),
    /// Declarative horizontal menu bar rendered and hit-tested in Rust.
    MenuBar(MenuBarWidget),
    /// Declarative status bar rendered in Rust (display-only).
    StatusBar(StatusBarWidget),
    /// Static dialog label rendered in Rust.
    Label(LabelWidget),
    /// Dialog push button rendered in Rust.
    Button(ButtonWidget),
    /// Single-line dialog text input rendered in Rust.
    InputLine(InputLineWidget),
    /// Dialog checkbox rendered in Rust.
    CheckBox(CheckBoxWidget),
    /// Dialog radio group rendered in Rust.
    RadioGroup(RadioGroupWidget),
    /// Scrolling list box.
    ListBox(ListBoxWidget),
    /// Standalone scroll bar.
    ScrollBar(ScrollBarWidget),
    /// Scrolling multi-line text view.
    ScrollView(ScrollViewWidget),
    /// Window or dialog frame with host-painted chrome.
    Frame(FrameWidget),
}

impl ViewWidget {
    /// Synchronize control paint flags with resolved retained view state.
    pub fn sync_view_state(&mut self, state: ViewState) {
        match self {
            Self::Label(widget) => widget.enabled = state.enabled,
            Self::Button(widget) => {
                widget.enabled = state.enabled;
                widget.focused = state.focused;
            }
            Self::InputLine(widget) => {
                widget.enabled = state.enabled;
                widget.focused = state.focused;
            }
            Self::CheckBox(widget) => {
                widget.enabled = state.enabled;
                widget.focused = state.focused;
            }
            Self::RadioGroup(widget) => {
                widget.enabled = state.enabled;
                widget.focused = state.focused;
            }
            Self::ListBox(widget) => {
                widget.enabled = state.enabled;
                widget.focused = state.focused;
            }
            Self::ScrollBar(widget) => {
                widget.enabled = state.enabled;
                widget.focused = state.focused;
            }
            Self::ScrollView(widget) => {
                widget.enabled = state.enabled;
                widget.focused = state.focused;
            }
            Self::Frame(widget) => widget.active = state.active,
            Self::SolidFill(_) | Self::MenuBar(_) | Self::StatusBar(_) => {}
        }
    }
    /// Return the stable introspection kind for this native widget.
    #[must_use]
    pub fn kind(&self) -> ViewKind {
        match self {
            Self::SolidFill(_) => ViewKind::SolidFill,
            Self::MenuBar(_) => ViewKind::MenuBar,
            Self::StatusBar(_) => ViewKind::StatusBar,
            Self::Label(_) => ViewKind::Label,
            Self::Button(_) => ViewKind::Button,
            Self::InputLine(_) => ViewKind::InputLine,
            Self::CheckBox(_) => ViewKind::CheckBox,
            Self::RadioGroup(_) => ViewKind::RadioGroup,
            Self::ListBox(_) => ViewKind::ListBox,
            Self::ScrollBar(_) => ViewKind::ScrollBar,
            Self::ScrollView(_) => ViewKind::ScrollView,
            Self::Frame(_) => ViewKind::Frame,
        }
    }

    /// Paint the widget into `rect`, clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        match self {
            Self::SolidFill(widget) => widget.paint(console, rect, damage),
            Self::MenuBar(widget) => widget.paint(console, rect, damage),
            Self::StatusBar(widget) => widget.clone().paint(console, rect, damage),
            Self::Label(widget) => widget.paint(console, rect, damage),
            Self::Button(widget) => widget.paint(console, rect, damage),
            Self::InputLine(widget) => widget.paint(console, rect, damage),
            Self::CheckBox(widget) => widget.paint(console, rect, damage),
            Self::RadioGroup(widget) => widget.paint(console, rect, damage),
            Self::ListBox(widget) => widget.paint(console, rect, damage),
            Self::ScrollBar(widget) => widget.paint(console, rect, damage),
            Self::ScrollView(widget) => widget.paint(console, rect, damage),
            Self::Frame(widget) => {
                widget.paint_underlay(console, rect, damage);
                widget.paint_overlay(console, rect, damage);
            }
        }
    }

    /// Paint the widget phase that precedes local handlers and descendants.
    pub fn paint_underlay(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        match self {
            Self::Frame(widget) => widget.paint_underlay(console, rect, damage),
            _ => self.paint(console, rect, damage),
        }
    }

    /// Paint chrome that must remain above local handlers and descendants.
    pub fn paint_overlay(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        if let Self::Frame(widget) = self {
            widget.paint_overlay(console, rect, damage);
        }
    }

    /// Paint menu popups after other widgets so pull-downs stay visible.
    pub fn paint_menu_overlays(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        if let Self::MenuBar(widget) = self {
            widget.paint_popup_overlay(console, rect, damage);
        }
    }

    /// Returns whether `damage` intersects any paintable region for this widget.
    #[must_use]
    pub fn intersects_damage(&self, rect: ViewRect, damage: DamageRegion) -> bool {
        match self {
            Self::MenuBar(widget) => widget
                .damage_rects(rect)
                .into_iter()
                .any(|region| intersects_damage_region(region, damage)),
            Self::SolidFill(_)
            | Self::StatusBar(_)
            | Self::Label(_)
            | Self::Button(_)
            | Self::InputLine(_)
            | Self::CheckBox(_)
            | Self::RadioGroup(_)
            | Self::ListBox(_)
            | Self::ScrollBar(_)
            | Self::ScrollView(_)
            | Self::Frame(_) => intersects_damage_region(rect, damage),
        }
    }

    /// Returns whether a mouse point hits this widget, including open menu popups.
    ///
    /// Mouse coordinates are one-based, matching `Std.Console.Event`.
    #[must_use]
    pub fn contains_point(&self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> bool {
        match self {
            Self::MenuBar(widget) => widget.contains_point(rect, mouse_x, mouse_y),
            Self::SolidFill(_)
            | Self::StatusBar(_)
            | Self::Label(_)
            | Self::Button(_)
            | Self::InputLine(_)
            | Self::CheckBox(_)
            | Self::RadioGroup(_)
            | Self::ListBox(_)
            | Self::ScrollBar(_)
            | Self::ScrollView(_)
            | Self::Frame(_) => rect.contains_console_mouse(mouse_x, mouse_y),
        }
    }
}

fn intersects_damage_region(rect: ViewRect, damage: DamageRegion) -> bool {
    match damage {
        DamageRegion::FullFrame => true,
        DamageRegion::Rect(dirty) => rect.intersects(dirty),
    }
}
