//! Screen-space hit targets for Turbo Vision headless `TestClickMouse`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/testing.md`

use turbo_vision::core::geometry::{Point, Rect};

/// Returns the screen-space bounds of a child widget inside a parent.
pub(in crate::vm::execute::io::tui::bridge) fn widget_screen_bounds(
    parent_bounds: Rect,
    local_bounds: Rect,
) -> Rect {
    Rect::new(
        parent_bounds.a.x + local_bounds.a.x,
        parent_bounds.a.y + local_bounds.a.y,
        parent_bounds.a.x + local_bounds.a.x + local_bounds.width(),
        parent_bounds.a.y + local_bounds.a.y + local_bounds.height(),
    )
}

/// Returns a desktop coordinate inside a child widget for headless mouse routing.
pub(in crate::vm::execute::io::tui::bridge) fn widget_mouse_click_point(
    parent_bounds: Rect,
    local_bounds: Rect,
) -> Point {
    let hit = widget_screen_bounds(parent_bounds, local_bounds);
    Point::new(
        hit.a.x + hit.width().saturating_sub(1) / 2,
        hit.a.y + hit.height().saturating_sub(1) / 2,
    )
}

/// Returns `true` when `(x, y)` lies inside `bounds`.
pub(in crate::vm::execute::io::tui::bridge) fn point_in_screen_bounds(
    bounds: Rect,
    x: i16,
    y: i16,
) -> bool {
    x >= bounds.a.x && x < bounds.b.x && y >= bounds.a.y && y < bounds.b.y
}
