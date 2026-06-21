//! Shared scrolling primitives for retained controls and frames.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

/// One-dimensional clamped scroll state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScrollModel {
    offset: usize,
    content_len: usize,
    viewport_len: usize,
}

impl ScrollModel {
    /// Create a model with fixed content and viewport extents.
    #[must_use]
    pub fn new(content_len: usize, viewport_len: usize) -> Self {
        Self {
            offset: 0,
            content_len,
            viewport_len,
        }
    }
    /// Return the first visible item index.
    #[must_use]
    pub const fn offset(self) -> usize {
        self.offset
    }
    /// Return the largest valid offset.
    #[must_use]
    pub fn max_offset(self) -> usize {
        self.content_len.saturating_sub(self.viewport_len)
    }
    /// Replace extents and clamp the offset.
    pub fn set_extents(&mut self, content_len: usize, viewport_len: usize) {
        self.content_len = content_len;
        self.viewport_len = viewport_len;
        self.offset = self.offset.min(self.max_offset());
    }
    /// Set a clamped offset.
    pub fn set_offset(&mut self, offset: usize) -> bool {
        let next = offset.min(self.max_offset());
        let changed = next != self.offset;
        self.offset = next;
        changed
    }
    /// Scroll by a signed item delta.
    pub fn scroll_by(&mut self, delta: i64) -> bool {
        let next = if delta < 0 {
            self.offset.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            self.offset.saturating_add(delta as usize)
        };
        self.set_offset(next)
    }
    /// Ensure an item is inside the viewport.
    pub fn ensure_visible(&mut self, index: usize) -> bool {
        let viewport = self.viewport_len.max(1);
        if index < self.offset {
            self.set_offset(index)
        } else if index >= self.offset.saturating_add(viewport) {
            self.set_offset(index.saturating_add(1).saturating_sub(viewport))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clamps_and_reveals() {
        let mut m = ScrollModel::new(10, 3);
        assert!(m.set_offset(99));
        assert_eq!(m.offset(), 7);
        assert!(m.ensure_visible(2));
        assert_eq!(m.offset(), 2);
    }
}
