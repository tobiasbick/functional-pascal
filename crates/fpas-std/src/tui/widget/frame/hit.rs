//! Frame chrome hit-testing for move and resize interaction.
//!
//! Plan: `docs/future/windows-dialogs/README.md`

use crate::ViewRect;

use super::{FrameGeometry, FrameResizeEdge};

/// Chrome hit result for pointer routing on a frame root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameChromeHit {
    /// Title-bar drag move.
    Move,
    /// Border or corner resize.
    Resize(FrameResizeEdge),
    /// No frame chrome interaction at this point.
    None,
}

/// Return whether `(x, y)` is on the draggable title-bar region.
#[must_use]
pub fn frame_title_drag_hit(geometry: &FrameGeometry, x: i64, y: i64) -> bool {
    if !geometry.title_bar.contains_point(x, y) {
        return false;
    }
    for slot in [
        geometry.buttons.close,
        geometry.buttons.zoom,
        geometry.buttons.zoom_back,
    ] {
        if slot.is_some_and(|rect| rect.contains_point(x, y)) {
            return false;
        }
    }
    true
}

/// Return a resize edge when `(x, y)` lies on the frame outer border.
#[must_use]
pub fn frame_resize_edge_at(outer: ViewRect, x: i64, y: i64) -> Option<FrameResizeEdge> {
    if !outer.contains_point(x, y) {
        return None;
    }

    let on_left = x == outer.x;
    let on_right = x == outer.x.saturating_add(outer.width.saturating_sub(1));
    let on_bottom = y == outer.y.saturating_add(outer.height.saturating_sub(1));

    match (on_left, on_right, on_bottom) {
        (true, _, true) => Some(FrameResizeEdge::SouthWest),
        (_, true, true) => Some(FrameResizeEdge::SouthEast),
        (_, true, _) => Some(FrameResizeEdge::East),
        (true, _, _) => Some(FrameResizeEdge::West),
        (_, _, true) => Some(FrameResizeEdge::South),
        _ => None,
    }
}

/// Resolve move vs resize priority for one frame root point.
#[must_use]
pub fn frame_chrome_hit(
    geometry: &FrameGeometry,
    movable: bool,
    resizable: bool,
    x: i64,
    y: i64,
) -> FrameChromeHit {
    if resizable
        && let Some(edge) = frame_resize_edge_at(geometry.outer, x, y)
        && !(movable && frame_title_drag_hit(geometry, x, y))
    {
        return FrameChromeHit::Resize(edge);
    }
    if movable && frame_title_drag_hit(geometry, x, y) {
        return FrameChromeHit::Move;
    }
    FrameChromeHit::None
}
