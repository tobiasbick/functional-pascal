//! Scrolling text view with an integrated vertical scroll bar.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use super::scroll_bar::{ScrollBarStyle, ScrollBarWidget};
use super::{paint_chars, truncated_chars};
use crate::{Console, DamageRegion, ScrollBarHit, ScrollBarOrientation, ScrollModel, ViewRect};

/// CRT colors for scroll views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollViewStyle {
    /// Background color.
    pub bg: u8,
    /// Text foreground color.
    pub fg: u8,
    /// Integrated scroll-bar style.
    pub scrollbar: ScrollBarStyle,
}
impl Default for ScrollViewStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 0,
            scrollbar: ScrollBarStyle::default(),
        }
    }
}

/// Multi-line read-only view with an integrated vertical scroll bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollViewWidget {
    /// Logical lines rendered in the content area.
    pub lines: Vec<String>,
    scroll: ScrollModel,
    /// Whether the control accepts interaction.
    pub enabled: bool,
    /// Whether the control owns keyboard focus.
    pub focused: bool,
    /// Current paint style.
    pub style: ScrollViewStyle,
    /// Active integrated scroll-bar thumb drag grab offset, if any.
    thumb_drag_grab: Option<usize>,
}

impl ScrollViewWidget {
    /// Create a scroll view sized for `viewport` visible rows.
    #[must_use]
    pub fn new(lines: Vec<String>, viewport: usize) -> Self {
        Self {
            scroll: ScrollModel::new(lines.len(), viewport),
            lines,
            enabled: true,
            focused: false,
            style: ScrollViewStyle::default(),
            thumb_drag_grab: None,
        }
    }

    /// Return whether a thumb drag is active on the integrated scroll bar.
    #[must_use]
    pub fn thumb_drag_active(&self) -> bool {
        self.thumb_drag_grab.is_some()
    }

    /// Return the scroll offset.
    #[must_use]
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Return the line count.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Replace lines and reset scroll.
    pub fn set_lines(&mut self, lines: Vec<String>, viewport: usize) {
        self.lines = lines;
        self.scroll = ScrollModel::new(self.lines.len(), viewport);
    }

    /// Scroll by a signed line delta.
    pub fn scroll_by(&mut self, delta: i64) -> bool {
        self.scroll.scroll_by(delta)
    }

    /// Scroll by one viewport page.
    pub fn scroll_page(&mut self, forward: bool) -> bool {
        self.scroll.scroll_page(forward)
    }

    /// Set a clamped scroll offset.
    pub fn set_offset(&mut self, offset: usize) -> bool {
        self.scroll.set_offset(offset)
    }

    /// Return the content rectangle excluding an integrated scroll bar when needed.
    #[must_use]
    pub fn content_rect(&self, rect: ViewRect) -> ViewRect {
        if self.scroll.needs_scroll() && rect.width > 1 {
            ViewRect {
                width: rect.width - 1,
                ..rect
            }
        } else {
            rect
        }
    }

    /// Return the integrated scroll-bar rectangle when scrolling is required.
    #[must_use]
    pub fn scrollbar_rect(&self, rect: ViewRect) -> Option<ViewRect> {
        if !self.scroll.needs_scroll() || rect.width <= 1 {
            return None;
        }
        Some(ViewRect {
            x: rect.x + rect.width - 1,
            y: rect.y,
            width: 1,
            height: rect.height,
        })
    }

    /// Resolve a mouse hit on the integrated scroll bar.
    #[must_use]
    pub fn scrollbar_hit(
        &self,
        rect: ViewRect,
        mouse_x: i64,
        mouse_y: i64,
    ) -> Option<ScrollBarHit> {
        let bar_rect = self.scrollbar_rect(rect)?;
        ScrollBarWidget::with_scroll(ScrollBarOrientation::Vertical, self.scroll)
            .hit_test(bar_rect, mouse_x, mouse_y)
    }

    /// Apply a scroll-bar hit on the integrated bar.
    pub fn apply_scrollbar_hit(&mut self, rect: ViewRect, hit: ScrollBarHit) -> bool {
        let _ = rect;
        match hit {
            ScrollBarHit::DecrementArrow => self.scroll_by(-1),
            ScrollBarHit::IncrementArrow => self.scroll_by(1),
            ScrollBarHit::TrackBefore => self.scroll_page(false),
            ScrollBarHit::TrackAfter => self.scroll_page(true),
            ScrollBarHit::Thumb => false,
        }
    }

    /// Begin a thumb drag on the integrated scroll bar.
    pub fn begin_thumb_drag(&mut self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> bool {
        let Some(bar_rect) = self.scrollbar_rect(rect) else {
            return false;
        };
        let bar = ScrollBarWidget::with_scroll(ScrollBarOrientation::Vertical, self.scroll);
        if bar.hit_test(bar_rect, mouse_x, mouse_y) != Some(ScrollBarHit::Thumb) {
            return false;
        }
        let mut drag_bar = bar;
        if !drag_bar.begin_thumb_drag(bar_rect, mouse_x, mouse_y) {
            return false;
        }
        self.thumb_drag_grab = drag_bar.thumb_drag_grab;
        true
    }

    /// Update scroll offset while dragging the integrated scroll-bar thumb.
    pub fn drag_thumb(&mut self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> bool {
        let Some(grab) = self.thumb_drag_grab else {
            return false;
        };
        let Some(bar_rect) = self.scrollbar_rect(rect) else {
            return false;
        };
        let mut bar = ScrollBarWidget::with_scroll(ScrollBarOrientation::Vertical, self.scroll);
        bar.thumb_drag_grab = Some(grab);
        if !bar.drag_thumb(bar_rect, mouse_x, mouse_y) {
            return false;
        }
        self.scroll = bar.scroll();
        true
    }

    /// End an active integrated scroll-bar thumb drag.
    pub fn end_thumb_drag(&mut self) {
        self.thumb_drag_grab = None;
    }

    /// Paint content and integrated scroll bar.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = damage.clip_rect(rect) else {
            return;
        };
        let content = self.content_rect(rect);
        console.fill_rect_crt(content, self.style.fg, self.style.bg, ' ');
        for (row, line) in self
            .lines
            .iter()
            .skip(self.scroll.offset())
            .take(content.height.max(0) as usize)
            .enumerate()
        {
            let line_rect = ViewRect {
                x: content.x,
                y: content.y + row as i64,
                width: content.width,
                height: 1,
            };
            paint_chars(
                console,
                line_rect,
                clip,
                truncated_chars(line, line_rect.width),
                |_| self.style.fg,
                self.style.bg,
            );
        }
        if let Some(bar_rect) = self.scrollbar_rect(rect) {
            let mut bar = ScrollBarWidget::with_scroll(ScrollBarOrientation::Vertical, self.scroll);
            bar.enabled = self.enabled;
            bar.focused = self.focused;
            bar.style = self.style.scrollbar;
            bar.paint(console, bar_rect, damage);
        }
    }
}
