//! Standalone vertical or horizontal scroll bar.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use super::{clip_rect_to_damage, paint_chars};
use crate::{
    Console, DamageRegion, ScrollBarHit, ScrollBarOrientation, ScrollBarThumb, ScrollModel,
    ViewRect, drag_offset, hit_zone, thumb_geometry, track_cells,
};

/// CRT colors for scroll bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollBarStyle {
    /// Background color.
    pub bg: u8,
    /// Track foreground color.
    pub fg: u8,
    /// Thumb foreground color.
    pub thumb_fg: u8,
    /// Arrow foreground color.
    pub arrow_fg: u8,
}
impl Default for ScrollBarStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 8,
            thumb_fg: 0,
            arrow_fg: 0,
        }
    }
}

/// Standalone scroll bar with Turbo Vision-style arrows and thumb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollBarWidget {
    orientation: ScrollBarOrientation,
    scroll: ScrollModel,
    /// Whether the control accepts interaction.
    pub enabled: bool,
    /// Whether the control owns keyboard focus.
    pub focused: bool,
    /// Current paint style.
    pub style: ScrollBarStyle,
    /// Active thumb drag grab offset inside the thumb, if any.
    pub(crate) thumb_drag_grab: Option<usize>,
}

impl ScrollBarWidget {
    /// Create a scroll bar with logical content and viewport lengths.
    #[must_use]
    pub fn new(orientation: ScrollBarOrientation, content_len: usize, viewport_len: usize) -> Self {
        Self {
            orientation,
            scroll: ScrollModel::new(content_len, viewport_len),
            enabled: true,
            focused: false,
            style: ScrollBarStyle::default(),
            thumb_drag_grab: None,
        }
    }

    /// Create a scroll bar sharing an existing scroll model.
    #[must_use]
    pub(crate) fn with_scroll(orientation: ScrollBarOrientation, scroll: ScrollModel) -> Self {
        Self {
            orientation,
            scroll,
            enabled: true,
            focused: false,
            style: ScrollBarStyle::default(),
            thumb_drag_grab: None,
        }
    }

    /// Return whether a thumb drag is active.
    #[must_use]
    pub fn thumb_drag_active(&self) -> bool {
        self.thumb_drag_grab.is_some()
    }

    /// Return the scroll model.
    #[must_use]
    pub const fn scroll(&self) -> ScrollModel {
        self.scroll
    }

    /// Return the scroll offset.
    #[must_use]
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Replace logical extents and clamp the offset.
    pub fn set_extents(&mut self, content_len: usize, viewport_len: usize) {
        self.scroll.set_extents(content_len, viewport_len);
    }

    /// Scroll by a signed item delta.
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

    /// Apply a mouse hit zone.
    pub fn apply_hit(&mut self, hit: ScrollBarHit) -> bool {
        match hit {
            ScrollBarHit::DecrementArrow => self.scroll_by(-1),
            ScrollBarHit::IncrementArrow => self.scroll_by(1),
            ScrollBarHit::TrackBefore => self.scroll_page(false),
            ScrollBarHit::TrackAfter => self.scroll_page(true),
            ScrollBarHit::Thumb => false,
        }
    }

    /// Begin a thumb drag when `mouse` hits the thumb. Returns the grab offset.
    pub fn begin_thumb_drag(&mut self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> bool {
        if self.hit_test(rect, mouse_x, mouse_y) != Some(ScrollBarHit::Thumb) {
            return false;
        }
        let Some((track, track_cell)) = self.track_cell_at(rect, mouse_x, mouse_y) else {
            return false;
        };
        let thumb = thumb_geometry(self.scroll, track);
        self.thumb_drag_grab = Some(track_cell.saturating_sub(thumb.start));
        true
    }

    /// Update scroll offset while dragging the thumb.
    pub fn drag_thumb(&mut self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> bool {
        let Some(grab) = self.thumb_drag_grab else {
            return false;
        };
        let Some((track, track_cell)) = self.track_cell_at_clamped(rect, mouse_x, mouse_y) else {
            return false;
        };
        self.set_offset(drag_offset(self.scroll, track, track_cell, grab))
    }

    /// End an active thumb drag.
    pub fn end_thumb_drag(&mut self) {
        self.thumb_drag_grab = None;
    }

    /// Resolve a mouse hit inside `rect`.
    #[must_use]
    pub fn hit_test(&self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> Option<ScrollBarHit> {
        if !rect.contains_console_mouse(mouse_x, mouse_y) {
            return None;
        }
        let bar_cells = match self.orientation {
            ScrollBarOrientation::Vertical => rect.height.max(0) as usize,
            ScrollBarOrientation::Horizontal => rect.width.max(0) as usize,
        };
        let cell = match self.orientation {
            ScrollBarOrientation::Vertical => {
                mouse_y.saturating_sub(1).saturating_sub(rect.y) as usize
            }
            ScrollBarOrientation::Horizontal => {
                mouse_x.saturating_sub(1).saturating_sub(rect.x) as usize
            }
        };
        hit_zone(self.scroll, bar_cells, cell)
    }

    fn track_cell_at(&self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> Option<(usize, usize)> {
        let (bar_cells, cell) = self.bar_cell(rect, mouse_x, mouse_y)?;
        if cell == 0 || cell + 1 == bar_cells {
            return None;
        }
        Some((track_cells(bar_cells), cell - 1))
    }

    fn track_cell_at_clamped(
        &self,
        rect: ViewRect,
        mouse_x: i64,
        mouse_y: i64,
    ) -> Option<(usize, usize)> {
        let (bar_cells, cell) = self.bar_cell(rect, mouse_x, mouse_y)?;
        let track = track_cells(bar_cells);
        if track == 0 {
            return None;
        }
        let track_cell = if cell == 0 {
            0
        } else if cell + 1 == bar_cells {
            track - 1
        } else {
            cell - 1
        };
        Some((track, track_cell))
    }

    fn bar_cell(&self, rect: ViewRect, mouse_x: i64, mouse_y: i64) -> Option<(usize, usize)> {
        let bar_cells = match self.orientation {
            ScrollBarOrientation::Vertical => rect.height.max(0) as usize,
            ScrollBarOrientation::Horizontal => rect.width.max(0) as usize,
        };
        if bar_cells < 3 {
            return None;
        }
        let cell = match self.orientation {
            ScrollBarOrientation::Vertical => {
                mouse_y.saturating_sub(1).saturating_sub(rect.y) as usize
            }
            ScrollBarOrientation::Horizontal => {
                mouse_x.saturating_sub(1).saturating_sub(rect.x) as usize
            }
        };
        (cell < bar_cells).then_some((bar_cells, cell))
    }

    /// Paint the scroll bar.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = clip_rect_to_damage(rect, damage) else {
            return;
        };
        console.fill_rect_crt(clip, self.style.fg, self.style.bg, ' ');
        let bar_cells = match self.orientation {
            ScrollBarOrientation::Vertical => rect.height.max(0) as usize,
            ScrollBarOrientation::Horizontal => rect.width.max(0) as usize,
        };
        if bar_cells < 3 {
            return;
        }
        let track = track_cells(bar_cells);
        let thumb = thumb_geometry(self.scroll, track);
        match self.orientation {
            ScrollBarOrientation::Vertical => self.paint_vertical(console, rect, clip, thumb),
            ScrollBarOrientation::Horizontal => self.paint_horizontal(console, rect, clip, thumb),
        }
    }

    fn paint_vertical(
        &self,
        console: &mut Console,
        rect: ViewRect,
        clip: ViewRect,
        thumb: ScrollBarThumb,
    ) {
        let height = rect.height.max(0) as usize;
        self.paint_cell(console, rect, clip, 0, '▲', self.style.arrow_fg);
        self.paint_cell(console, rect, clip, height - 1, '▼', self.style.arrow_fg);
        for row in 1..height.saturating_sub(1) {
            let track_row = row - 1;
            let ch = if track_row >= thumb.start && track_row < thumb.start + thumb.size {
                '█'
            } else {
                '░'
            };
            let fg = if ch == '█' {
                self.style.thumb_fg
            } else {
                self.style.fg
            };
            self.paint_cell(console, rect, clip, row, ch, fg);
        }
    }

    fn paint_horizontal(
        &self,
        console: &mut Console,
        rect: ViewRect,
        clip: ViewRect,
        thumb: ScrollBarThumb,
    ) {
        let width = rect.width.max(0) as usize;
        self.paint_cell_at(console, rect, clip, 0, rect.y, '◄', self.style.arrow_fg);
        self.paint_cell_at(
            console,
            rect,
            clip,
            width.saturating_sub(1),
            rect.y,
            '►',
            self.style.arrow_fg,
        );
        for col in 1..width.saturating_sub(1) {
            let track_col = col - 1;
            let ch = if track_col >= thumb.start && track_col < thumb.start + thumb.size {
                '█'
            } else {
                '░'
            };
            let fg = if ch == '█' {
                self.style.thumb_fg
            } else {
                self.style.fg
            };
            self.paint_cell_at(console, rect, clip, col, rect.y, ch, fg);
        }
    }

    fn paint_cell(
        &self,
        console: &mut Console,
        rect: ViewRect,
        clip: ViewRect,
        row: usize,
        ch: char,
        fg: u8,
    ) {
        self.paint_cell_at(console, rect, clip, 0, rect.y + row as i64, ch, fg);
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_cell_at(
        &self,
        console: &mut Console,
        rect: ViewRect,
        clip: ViewRect,
        col: usize,
        y: i64,
        ch: char,
        fg: u8,
    ) {
        let cell = ViewRect {
            x: rect.x + col as i64,
            y,
            width: 1,
            height: 1,
        };
        paint_chars(
            console,
            cell,
            clip,
            std::iter::once((0, ch)),
            |_| fg,
            self.style.bg,
        );
    }
}
