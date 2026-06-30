//! Rust-hosted TUI widgets painted directly by the VM host.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

mod control;
mod solid_fill;
mod status_bar;

pub use control::ScrollBarStyle;
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
    /// Declarative status bar rendered in Rust (display-only).
    StatusBar(StatusBarWidget),
}

impl ViewWidget {
    /// Synchronize control paint flags with resolved retained view state.
    pub fn sync_view_state(&mut self, _state: ViewState) {}

    /// Return the stable introspection kind for this native widget.
    #[must_use]
    pub fn kind(&self) -> ViewKind {
        match self {
            Self::SolidFill(_) => ViewKind::SolidFill,
            Self::StatusBar(_) => ViewKind::StatusBar,
        }
    }

    /// Paint the widget into `rect`, clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        match self {
            Self::SolidFill(widget) => widget.paint(console, rect, damage),
            Self::StatusBar(widget) => widget.paint(console, rect, damage),
        }
    }

    /// Paint the widget phase that precedes local handlers and descendants.
    pub fn paint_underlay(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        self.paint(console, rect, damage);
    }

    /// Paint chrome that must remain above local handlers and descendants.
    pub fn paint_overlay(&self, _console: &mut Console, _rect: ViewRect, _damage: DamageRegion) {}

    /// Paint menu popups after other widgets so pull-downs stay visible.
    pub fn paint_scene_overlay(
        &self,
        _console: &mut Console,
        _rect: ViewRect,
        _damage: DamageRegion,
    ) {
    }

    /// Return whether this widget contributes a paint layer above the retained scene.
    #[must_use]
    pub fn has_scene_overlay(&self) -> bool {
        false
    }

    /// Returns whether `damage` intersects any paintable region for this widget.
    #[must_use]
    pub fn intersects_damage(&self, rect: ViewRect, damage: DamageRegion) -> bool {
        damage.intersects_rect(rect)
    }

    /// Returns whether a mouse point hits this widget, including open menu popups.
    ///
    /// Mouse coordinates are one-based, matching `Std.Console.Event`.
    #[must_use]
    pub fn contains_point(&self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> bool {
        rect.contains_console_mouse(mouse_x, mouse_y)
    }
}
