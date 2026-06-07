//! Rust-hosted TUI widgets painted directly by the VM host.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui-app.md`

mod menu_bar;
mod menu_popup;
mod solid_fill;
mod status_bar;

pub use menu_bar::{MenuBarItem, MenuBarMouseResult, MenuBarStyle, MenuBarWidget};
pub use menu_popup::MenuPopupItem;
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
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        match self {
            Self::SolidFill(widget) => widget.paint(console, rect, damage),
            Self::MenuBar(widget) => widget.paint(console, rect, damage),
            Self::StatusBar(widget) => widget.clone().paint(console, rect, damage),
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
            Self::SolidFill(_) | Self::StatusBar(_) => intersects_damage_region(rect, damage),
        }
    }

    /// Returns whether a mouse point hits this widget, including open menu popups.
    ///
    /// Mouse coordinates are one-based, matching `Std.Console.Event`.
    #[must_use]
    pub fn contains_point(&self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> bool {
        match self {
            Self::MenuBar(widget) => widget.contains_point(rect, mouse_x, mouse_y),
            Self::SolidFill(_) | Self::StatusBar(_) => rect.contains_console_mouse(mouse_x, mouse_y),
        }
    }
}

fn intersects_damage_region(rect: ViewRect, damage: DamageRegion) -> bool {
    match damage {
        DamageRegion::FullFrame => true,
        DamageRegion::Rect(dirty) => {
            let left_right = rect.x.saturating_add(rect.width);
            let left_bottom = rect.y.saturating_add(rect.height);
            let right_right = dirty.x.saturating_add(dirty.width);
            let right_bottom = dirty.y.saturating_add(dirty.height);

            rect.x < right_right
                && dirty.x < left_right
                && rect.y < right_bottom
                && dirty.y < left_bottom
        }
    }
}
