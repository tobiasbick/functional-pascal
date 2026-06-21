//! Unit tests for scroll model and geometry.

use super::{ScrollBarHit, ScrollModel, hit_zone, thumb_geometry, track_cells};

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
