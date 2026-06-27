//! Frame chrome hit-testing for move and resize interaction.
//!
//! Plan: `docs/future/tui/completed.md`

use crate::ViewRect;

use super::{FrameGeometry, FrameResizeEdge};

/// Chrome hit result for pointer routing on a frame root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameChromeHit {
    /// Title-bar close button.
    Close,
    /// Title-bar zoom button.
    Zoom,
    /// Title-bar restore button.
    ZoomBack,
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

/// Resolve title-bar buttons, move, or resize priority for one frame root point.
#[must_use]
pub fn frame_chrome_hit(
    geometry: &FrameGeometry,
    capabilities: super::FrameCapabilities,
    zoomed: bool,
    x: i64,
    y: i64,
) -> FrameChromeHit {
    if capabilities.closable
        && geometry
            .buttons
            .close
            .is_some_and(|rect| rect.contains_point(x, y))
    {
        return FrameChromeHit::Close;
    }
    if capabilities.zoomable
        && !zoomed
        && geometry
            .buttons
            .zoom
            .is_some_and(|rect| rect.contains_point(x, y))
    {
        return FrameChromeHit::Zoom;
    }
    if capabilities.zoomable
        && zoomed
        && geometry
            .buttons
            .zoom_back
            .is_some_and(|rect| rect.contains_point(x, y))
    {
        return FrameChromeHit::ZoomBack;
    }
    if capabilities.resizable
        && let Some(edge) = frame_resize_edge_at(geometry.outer, x, y)
        && !(capabilities.movable && frame_title_drag_hit(geometry, x, y))
    {
        return FrameChromeHit::Resize(edge);
    }
    if capabilities.movable && frame_title_drag_hit(geometry, x, y) {
        return FrameChromeHit::Move;
    }
    FrameChromeHit::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widget::frame::{FrameCapabilities, FrameContentSize, FrameGeometry};

    fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width,
            height,
        }
    }

    fn chrome_geometry() -> FrameGeometry {
        FrameGeometry::resolve(
            rect(2, 1, 16, 6),
            FrameContentSize::new(0, 0),
            FrameCapabilities {
                movable: true,
                resizable: true,
                closable: true,
                zoomable: true,
                scrollable: false,
            },
        )
        .expect("valid frame")
    }

    #[test]
    fn frame_chrome_hit_detects_close_and_zoom_buttons() {
        let geometry = chrome_geometry();
        assert_eq!(
            frame_chrome_hit(
                &geometry,
                FrameCapabilities {
                    movable: true,
                    resizable: true,
                    closable: true,
                    zoomable: true,
                    scrollable: false,
                },
                false,
                4,
                1
            ),
            FrameChromeHit::Close
        );
        assert_eq!(
            frame_chrome_hit(
                &geometry,
                FrameCapabilities {
                    movable: true,
                    resizable: true,
                    closable: true,
                    zoomable: true,
                    scrollable: false,
                },
                false,
                14,
                1
            ),
            FrameChromeHit::Zoom
        );
        assert_eq!(
            frame_chrome_hit(
                &geometry,
                FrameCapabilities {
                    movable: true,
                    resizable: true,
                    closable: true,
                    zoomable: true,
                    scrollable: false,
                },
                true,
                15,
                1
            ),
            FrameChromeHit::ZoomBack
        );
    }
}
