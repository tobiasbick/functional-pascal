//! Rust-internal redraw damage tracking for `Std.Tui`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (public contract),
//! `docs/future/tui-application-framework.md` (Phase 7 performance plan).

use crate::ViewRect;

/// Pending redraw scope for the hosted TUI application surface.
///
/// This is a Rust-host detail used by `fpas-vm` while Phase 7 performance work is in
/// progress. It is not part of the FPAS language surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageRegion {
    /// The next paint must redraw the entire application surface.
    FullFrame,
    /// The next paint may restrict itself to the dirty rectangle.
    Rect(ViewRect),
}

impl DamageRegion {
    /// Clips `rect` to the dirty region; returns `None` when there is no overlap.
    #[must_use]
    pub(crate) fn clip_rect(self, rect: ViewRect) -> Option<ViewRect> {
        match self {
            Self::FullFrame => Some(rect),
            Self::Rect(dirty) => rect.intersection(dirty),
        }
    }

    /// Returns whether `rect` overlaps the dirty region.
    #[must_use]
    pub(crate) fn intersects_rect(self, rect: ViewRect) -> bool {
        match self {
            Self::FullFrame => true,
            Self::Rect(dirty) => rect.intersects(dirty),
        }
    }
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
    pub(crate) fn mark_rect(&mut self, rect: ViewRect) {
        if rect.width <= 0 || rect.height <= 0 {
            return;
        }

        self.pending = Some(match self.pending {
            Some(DamageRegion::FullFrame) => DamageRegion::FullFrame,
            Some(DamageRegion::Rect(existing)) => DamageRegion::Rect(existing.union(rect)),
            None => DamageRegion::Rect(rect),
        });
    }

    /// Returns `true` when any redraw work is pending.
    #[must_use]
    pub(crate) fn has_damage(&self) -> bool {
        self.pending.is_some()
    }

    /// Returns the pending damage without clearing it.
    #[must_use]
    pub(crate) fn peek(&self) -> Option<DamageRegion> {
        self.pending
    }

    /// Consumes and returns the pending damage description.
    pub(crate) fn take(&mut self) -> Option<DamageRegion> {
        self.pending.take()
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
    fn damage_region_clip_and_intersect() {
        let rect = ViewRect {
            x: 5,
            y: 5,
            width: 10,
            height: 4,
        };
        let dirty = ViewRect {
            x: 8,
            y: 3,
            width: 4,
            height: 6,
        };

        assert_eq!(
            DamageRegion::Rect(dirty).clip_rect(rect),
            Some(ViewRect {
                x: 8,
                y: 5,
                width: 4,
                height: 4,
            })
        );
        assert!(DamageRegion::Rect(dirty).intersects_rect(rect));
        assert!(
            !DamageRegion::Rect(ViewRect {
                x: 20,
                y: 20,
                width: 1,
                height: 1,
            })
            .intersects_rect(rect)
        );
        assert_eq!(DamageRegion::FullFrame.clip_rect(rect), Some(rect));
        assert!(DamageRegion::FullFrame.intersects_rect(rect));
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
