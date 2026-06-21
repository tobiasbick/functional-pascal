//! Rust-internal view registry for the TUI application framework (Phase 7).
//!
//! View handles are opaque identifiers maintained entirely by the host. The registry tracks a
//! small view tree: root views use absolute terminal coordinates, child views use coordinates
//! relative to their parent, and sibling order defines paint and hit-test z-order.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui/app/README.md`

mod activation;
mod desktop;
mod focus;
mod geometry;
mod introspection;
mod routing;
mod state;
mod tree;

use std::collections::HashMap;

use super::widget::frame::{FrameRootState, WindowInteraction};

#[cfg(test)]
mod tests;

pub use activation::RootActivation;
pub use desktop::{DesktopMetrics, WindowPalette, WindowShadow};
pub use introspection::{TUI_VIEW_KIND_VARIANTS, ViewKind};
pub use routing::{EventOutcome, EventPhase, EventRoute, RoutedEvent};
pub use state::{ResolvedView, ViewOptions, ViewState};

/// Opaque handle identifying a host-managed view.
///
/// FPAS still treats this as an integer token on the Pascal side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(u32);

impl ViewId {
    /// Construct a view id from its raw host representation.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the raw host representation used in the VM bridge today.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Axis-aligned bounding box for a view in terminal-cell coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewRect {
    /// Left edge in terminal cells.
    pub x: i64,
    /// Top edge in terminal cells.
    pub y: i64,
    /// Width in terminal cells.
    pub width: i64,
    /// Height in terminal cells.
    pub height: i64,
}

impl ViewRect {
    /// Return `true` when this rectangle covers no terminal cells.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    /// Return the bounding rectangle containing `self` and `other`.
    ///
    /// Both rectangles must be non-empty.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        debug_assert!(!self.is_empty());
        debug_assert!(!other.is_empty());

        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        }
    }

    /// Return the overlapping terminal-cell rectangle, if any.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        (right > x && bottom > y).then_some(Self {
            x,
            y,
            width: right - x,
            height: bottom - y,
        })
    }

    /// Return whether this rectangle overlaps `other`.
    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        self.intersection(other).is_some()
    }

    /// Return `true` when terminal-cell position `(x, y)` is inside this rectangle.
    ///
    /// View rectangles use zero-based coordinates (`0` is the top-left cell).
    #[must_use]
    pub fn contains_point(self, x: i64, y: i64) -> bool {
        let max_x = self.x.saturating_add(self.width.max(0));
        let max_y = self.y.saturating_add(self.height.max(0));
        x >= self.x && y >= self.y && x < max_x && y < max_y
    }

    /// Hit-test using one-based coordinates from `Std.Console.Event` mouse fields.
    #[must_use]
    pub fn contains_console_mouse(self, mouse_x: i64, mouse_y: i64) -> bool {
        self.contains_point(mouse_x.saturating_sub(1), mouse_y.saturating_sub(1))
    }

    fn right(self) -> i64 {
        self.x.saturating_add(self.width.max(0))
    }

    fn bottom(self) -> i64 {
        self.y.saturating_add(self.height.max(0))
    }
}

#[derive(Debug)]
struct ViewEntry {
    id: ViewId,
    local_rect: ViewRect,
    parent: Option<ViewId>,
    children: Vec<ViewId>,
    current_child: Option<ViewId>,
    state: ViewState,
    options: ViewOptions,
}

/// Host-side registry for all active views in a TUI session.
///
/// Root views use absolute terminal coordinates. Child views use coordinates relative to their
/// parent. Sibling insertion order defines z-order: later siblings paint later and therefore sit
/// on top during hit-testing.
///
/// Focus traversal is derived from tree order and each view's [`ViewOptions`].
#[derive(Debug, Default)]
pub struct ViewRegistry {
    next_id: u32,
    views: Vec<ViewEntry>,
    roots: Vec<ViewId>,
    focused: Option<ViewId>,
    pointer_capture: Option<ViewId>,
    desktop: DesktopMetrics,
    pub(crate) frame_roots: HashMap<ViewId, FrameRootState>,
    pub(crate) window_interaction: Option<WindowInteraction>,
}
