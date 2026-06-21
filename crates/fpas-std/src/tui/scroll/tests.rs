//! Unit tests for scroll model and geometry.

use super::{ScrollBarHit, ScrollModel, drag_offset, hit_zone, thumb_geometry, track_cells};
use crate::{ScrollBarOrientation, ScrollBarWidget, ViewRect};

#[test]
fn scroll_model_clamps_offset() {
    let mut model = ScrollModel::new(10, 3);
    assert!(model.set_offset(99));
    assert_eq!(model.offset(), 7);
    assert!(model.scroll_by(-99));
    assert_eq!(model.offset(), 0);
}

#[test]
fn scroll_model_pages() {
    let mut model = ScrollModel::new(20, 4);
    assert!(model.scroll_page(true));
    assert_eq!(model.offset(), 4);
    assert!(model.scroll_page(false));
    assert_eq!(model.offset(), 0);
}

#[test]
fn thumb_geometry_fills_track_when_not_needed() {
    let model = ScrollModel::new(3, 5);
    let thumb = thumb_geometry(model, track_cells(8));
    assert_eq!(thumb, super::ScrollBarThumb { start: 0, size: 6 });
}

#[test]
fn hit_zone_maps_track_cells() {
    let model = ScrollModel::new(20, 4);
    let bar = 8;
    assert_eq!(hit_zone(model, bar, 0), Some(ScrollBarHit::DecrementArrow));
    assert_eq!(hit_zone(model, bar, 7), Some(ScrollBarHit::IncrementArrow));
    assert!(matches!(
        hit_zone(model, bar, 1),
        Some(ScrollBarHit::TrackBefore | ScrollBarHit::Thumb)
    ));
}

#[test]
fn drag_offset_maps_track_cell_to_offset() {
    let scroll = ScrollModel::new(20, 4);
    assert_eq!(drag_offset(scroll, 6, 4, 0), 12);
}

#[test]
fn scroll_bar_thumb_drag_updates_offset() {
    let mut bar = ScrollBarWidget::new(ScrollBarOrientation::Vertical, 20, 4);
    let rect = ViewRect {
        x: 0,
        y: 0,
        width: 1,
        height: 8,
    };
    assert!(bar.begin_thumb_drag(rect, 1, 2));
    assert!(bar.drag_thumb(rect, 1, 6));
    assert_eq!(bar.scroll_offset(), 12);
    bar.end_thumb_drag();
    assert!(!bar.thumb_drag_active());
}
