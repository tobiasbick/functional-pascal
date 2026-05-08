//! Rust-internal redraw damage tracking for `Std.Tui`.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (public contract),
//! `docs/future/tui-application-framework.md` (Phase 7 performance plan).

use crate::ViewRect;

/// Pending redraw scope for the hosted TUI application surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DamageRegion {
    /// The next paint must redraw the entire application surface.
    FullFrame,
    /// The next paint may restrict itself to the dirty rectangle.
    #[allow(
        dead_code,
        reason = "Phase 7 groundwork adds rectangle damage accumulation before the host emits partial invalidations"
    )]
    Rect(ViewRect),
}

/// Accumulates dirty regions until the host consumes them before `OnPaint`.
#[derive(Debug, Default)]
pub(crate) struct DamageTracker {
    pending: Option<DamageRegion>,
}

impl DamageTracker {
    /// Clears all pending damage.
    pub(crate) fn clear(&mut self) {
        self.pending = None;
    }

    /// Marks the full frame dirty.
    pub(crate) fn mark_full(&mut self) {
        self.pending = Some(DamageRegion::FullFrame);
    }

    /// Merges a dirty rectangle into the pending damage set.
    ///
    /// Non-positive rectangles are ignored because they do not cover visible cells.
    #[allow(
        dead_code,
        reason = "Phase 7 groundwork adds rectangle damage accumulation before the host emits partial invalidations"
    )]
    pub(crate) fn mark_rect(&mut self, rect: ViewRect) {
        if rect.width <= 0 || rect.height <= 0 {
            return;
        }

        self.pending = Some(match self.pending {
            Some(DamageRegion::FullFrame) => DamageRegion::FullFrame,
            Some(DamageRegion::Rect(existing)) => DamageRegion::Rect(union_rects(existing, rect)),
            None => DamageRegion::Rect(rect),
        });
    }

    /// Returns `true` when any redraw work is pending.
    #[must_use]
    pub(crate) fn has_damage(&self) -> bool {
        self.pending.is_some()
    }

    /// Consumes and returns the pending damage description.
    pub(crate) fn take(&mut self) -> Option<DamageRegion> {
        self.pending.take()
    }
}

#[allow(
    dead_code,
    reason = "Phase 7 groundwork adds rectangle damage accumulation before the host emits partial invalidations"
)]
fn union_rects(left: ViewRect, right: ViewRect) -> ViewRect {
    let min_x = left.x.min(right.x);
    let min_y = left.y.min(right.y);
    let max_x = left
        .x
        .saturating_add(left.width)
        .max(right.x.saturating_add(right.width));
    let max_y = left
        .y
        .saturating_add(left.height)
        .max(right.y.saturating_add(right.height));

    ViewRect {
        x: min_x,
        y: min_y,
        width: max_x.saturating_sub(min_x),
        height: max_y.saturating_sub(min_y),
    }
}

#[cfg(test)]
mod tests {
    use super::{DamageRegion, DamageTracker};
    use crate::ViewRect;

    #[test]
    fn damage_tracker_merges_overlapping_rectangles() {
        let mut tracker = DamageTracker::default();
        tracker.mark_rect(ViewRect {
            x: 2,
            y: 3,
            width: 4,
            height: 5,
        });
        tracker.mark_rect(ViewRect {
            x: 5,
            y: 1,
            width: 3,
            height: 4,
        });

        assert_eq!(
            tracker.take(),
            Some(DamageRegion::Rect(ViewRect {
                x: 2,
                y: 1,
                width: 6,
                height: 7,
            }))
        );
    }

    #[test]
    fn damage_tracker_full_frame_dominates_rectangles() {
        let mut tracker = DamageTracker::default();
        tracker.mark_rect(ViewRect {
            x: 10,
            y: 10,
            width: 2,
            height: 2,
        });
        tracker.mark_full();
        tracker.mark_rect(ViewRect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        });

        assert_eq!(tracker.take(), Some(DamageRegion::FullFrame));
    }

    #[test]
    fn damage_tracker_ignores_non_positive_rectangles() {
        let mut tracker = DamageTracker::default();

        tracker.mark_rect(ViewRect {
            x: 1,
            y: 1,
            width: 0,
            height: 4,
        });
        tracker.mark_rect(ViewRect {
            x: 1,
            y: 1,
            width: 4,
            height: -1,
        });

        assert!(!tracker.has_damage());
    }
}
