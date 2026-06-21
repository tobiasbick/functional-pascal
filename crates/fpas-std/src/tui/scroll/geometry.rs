//! Shared scroll-bar track and thumb geometry.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use super::ScrollModel;

/// Scroll-bar axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBarOrientation {
    /// Vertical `▲█▼` bar.
    Vertical,
    /// Horizontal `◄█►` bar.
    Horizontal,
}

/// Resolved thumb placement inside a scroll-bar track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollBarThumb {
    /// Zero-based track row/column where the thumb starts.
    pub start: usize,
    /// Thumb length in track cells.
    pub size: usize,
}

/// Mouse hit zones inside a scroll bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBarHit {
    /// Top/left arrow cell.
    DecrementArrow,
    /// Bottom/right arrow cell.
    IncrementArrow,
    /// Track before the thumb.
    TrackBefore,
    /// Thumb cell.
    Thumb,
    /// Track after the thumb.
    TrackAfter,
}

/// Compute the track length after reserving arrow cells.
#[must_use]
pub fn track_cells(bar_cells: usize) -> usize {
    bar_cells.saturating_sub(2)
}

/// Resolve thumb placement for one scroll model and track length.
#[must_use]
pub fn thumb_geometry(scroll: ScrollModel, track: usize) -> ScrollBarThumb {
    if track == 0 {
        return ScrollBarThumb { start: 0, size: 0 };
    }
    if !scroll.needs_scroll() {
        return ScrollBarThumb {
            start: 0,
            size: track,
        };
    }
    let max_offset = scroll.max_offset().max(1);
    let size = thumb_size(scroll, track);
    let travel = track.saturating_sub(size);
    let start = scroll.offset().saturating_mul(travel) / max_offset;
    ScrollBarThumb { start, size }
}

/// Return the thumb length for one track.
#[must_use]
pub fn thumb_size(scroll: ScrollModel, track: usize) -> usize {
    if track == 0 {
        return 0;
    }
    if !scroll.needs_scroll() {
        return track;
    }
    (scroll.viewport_len().saturating_mul(track) / scroll.content_len().max(1)).max(1)
}

/// Map a thumb start track cell to a scroll offset.
#[must_use]
pub fn offset_from_thumb_start(scroll: ScrollModel, track: usize, thumb_start: usize) -> usize {
    if track == 0 || !scroll.needs_scroll() {
        return 0;
    }
    let size = thumb_size(scroll, track);
    let travel = track.saturating_sub(size);
    if travel == 0 {
        return 0;
    }
    let thumb_start = thumb_start.min(travel);
    thumb_start.saturating_mul(scroll.max_offset()) / travel
}

/// Resolve scroll offset while dragging with a fixed grab point inside the thumb.
#[must_use]
pub fn drag_offset(scroll: ScrollModel, track: usize, track_cell: usize, grab: usize) -> usize {
    if track == 0 || !scroll.needs_scroll() {
        return 0;
    }
    let size = thumb_size(scroll, track);
    let travel = track.saturating_sub(size);
    if travel == 0 {
        return 0;
    }
    let thumb_start = track_cell.saturating_sub(grab).min(travel);
    offset_from_thumb_start(scroll, track, thumb_start)
}

/// Map a zero-based cell inside the bar to a hit zone.
#[must_use]
pub fn hit_zone(scroll: ScrollModel, bar_cells: usize, cell: usize) -> Option<ScrollBarHit> {
    if bar_cells < 3 {
        return None;
    }
    if cell == 0 {
        return Some(ScrollBarHit::DecrementArrow);
    }
    if cell + 1 == bar_cells {
        return Some(ScrollBarHit::IncrementArrow);
    }
    let track = track_cells(bar_cells);
    let thumb = thumb_geometry(scroll, track);
    let track_cell = cell - 1;
    if track_cell < thumb.start {
        Some(ScrollBarHit::TrackBefore)
    } else if track_cell < thumb.start.saturating_add(thumb.size) {
        Some(ScrollBarHit::Thumb)
    } else {
        Some(ScrollBarHit::TrackAfter)
    }
}
