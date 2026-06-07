//! Rust-hosted TUI widgets painted directly by the VM host.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui-app.md`

mod menu_bar;
mod solid_fill;
mod status_bar;

pub use menu_bar::{MenuBarItem, MenuBarMouseResult, MenuBarStyle, MenuBarWidget};
pub use solid_fill::SolidFillWidget;
pub use status_bar::{StatusBarSegment, StatusBarStyle, StatusBarWidget};

use crate::{Console, DamageRegion, ViewRect};

/// Native widget attached to a host-managed view.
///
/// Spec: `docs/pascal/std/tui-app.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewWidget {
    /// Solid CRT-color fill, optionally tiled with one character.
    SolidFill(SolidFillWidget),
    /// Declarative horizontal menu bar rendered and hit-tested in Rust.
    MenuBar(MenuBarWidget),
    /// Declarative status bar rendered in Rust (display-only).
    StatusBar(StatusBarWidget),
}

impl ViewWidget {
    /// Paint the widget into `rect`, clipped to `damage`.
    pub fn paint(self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        match self {
            Self::SolidFill(widget) => widget.paint(console, rect, damage),
            Self::MenuBar(widget) => widget.paint(console, rect, damage),
            Self::StatusBar(widget) => widget.paint(console, rect, damage),
        }
    }
}
