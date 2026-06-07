//! Rust-internal view registry for the TUI application framework (Phase 7).
//!
//! View handles are opaque identifiers maintained entirely by the host. The registry tracks a
//! small view tree: root views use absolute terminal coordinates, child views use coordinates
//! relative to their parent, and sibling order defines paint and hit-test z-order.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui-app.md`

mod focus;
mod tree;

#[cfg(test)]
mod tests;

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
}

#[derive(Debug)]
struct ViewEntry {
    id: ViewId,
    local_rect: ViewRect,
    parent: Option<ViewId>,
    children: Vec<ViewId>,
}

/// Host-side registry for all active views in a TUI session.
///
/// Root views use absolute terminal coordinates. Child views use coordinates relative to their
/// parent. Sibling insertion order defines z-order: later siblings paint later and therefore sit
/// on top during hit-testing.
///
/// The focus chain remains a separate ordered list of [`ViewId`]s. Adding a view to that chain is
/// still explicit via [`push_child`][Self::push_child].
#[derive(Debug, Default)]
pub struct ViewRegistry {
    next_id: u32,
    views: Vec<ViewEntry>,
    roots: Vec<ViewId>,
    children: Vec<ViewId>,
    focused: Option<usize>,
}
