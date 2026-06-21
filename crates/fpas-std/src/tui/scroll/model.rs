//! One-dimensional clamped scroll offsets shared by controls and frame chrome.
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

    /// Return the logical content length.
    #[must_use]
    pub const fn content_len(self) -> usize {
        self.content_len
    }

    /// Return the visible viewport length.
    #[must_use]
    pub const fn viewport_len(self) -> usize {
        self.viewport_len
    }

    /// Return whether scrolling is required.
    #[must_use]
    pub fn needs_scroll(self) -> bool {
        self.content_len > self.viewport_len.max(1)
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

    /// Scroll by one viewport page toward the start or end.
    pub fn scroll_page(&mut self, forward: bool) -> bool {
        let page = self.viewport_len.max(1);
        self.scroll_by(if forward { page as i64 } else { -(page as i64) })
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
