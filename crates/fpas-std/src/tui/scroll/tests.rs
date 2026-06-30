//! Unit tests for scroll model and geometry.

use super::geometry::offset_from_thumb_start;
use super::{ScrollBarHit, ScrollModel, drag_offset, hit_zone, thumb_geometry, track_cells};
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
fn thumb_geometry_preserves_bounds_for_edge_extents() {
    let extent_cases = [
        (0, 0),
        (0, 4),
        (4, 4),
        (4, 8),
        (20, 4),
        (1_000_000, 1),
        (1_000_000, 999_999),
    ];
    let track_cases = [0, 1, 2, 6, 80];

    for (content_len, viewport_len) in extent_cases {
        let base = ScrollModel::new(content_len, viewport_len);
        let max_offset = base.max_offset();
        let offset_cases = [
            0,
            max_offset / 2,
            max_offset,
            max_offset.saturating_add(100),
        ];

        for requested_offset in offset_cases {
            let mut scroll = base;
            scroll.set_offset(requested_offset);

            for track in track_cases {
                let thumb = thumb_geometry(scroll, track);
                let end = thumb.start.checked_add(thumb.size).expect("thumb end");
                assert!(end <= track, "thumb {thumb:?} must fit track {track}");

                if track == 0 {
                    assert_eq!(thumb.start, 0);
                    assert_eq!(thumb.size, 0);
                } else if scroll.needs_scroll() {
                    assert!(thumb.size >= 1, "scrolling thumb must remain visible");
                } else {
                    assert_eq!(thumb.start, 0);
                    assert_eq!(thumb.size, track);
                }

                for thumb_start in [0, track / 2, track, track.saturating_add(100)] {
                    let offset = offset_from_thumb_start(scroll, track, thumb_start);
                    assert!(offset <= scroll.max_offset(), "offset {offset} exceeds max");
                }
            }
        }
    }
}
