//! Frame-integrated scroll chrome paint, hit-testing, and offset state.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::{
    Console, DamageRegion, ScrollBarHit, ScrollBarOrientation, ScrollBarStyle, ScrollBarThumb,
    ScrollModel, ViewId, ViewRect, ViewRegistry, drag_offset, hit_zone, thumb_geometry,
    track_cells,
};

use super::state::FrameScrollInteraction;
use super::{FrameContentSize, FrameGeometry, FrameRootState, FrameStyle};

/// Query state for one scrollable frame root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameScrollState {
    /// Horizontal scroll offset in terminal cells.
    pub offset_x: i64,
    /// Vertical scroll offset in terminal cells.
    pub offset_y: i64,
    /// Logical content width in terminal cells.
    pub content_width: i64,
    /// Logical content height in terminal cells.
    pub content_height: i64,
}

/// Scroll-bar hit on frame-owned chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameScrollHit {
    /// Vertical `▲█▼` bar.
    Vertical(ScrollBarHit),
    /// Horizontal `◄█►` bar.
    Horizontal(ScrollBarHit),
}

/// Paint frame-owned scroll bars after border chrome.
pub fn paint_scrollbars(
    console: &mut Console,
    geometry: FrameGeometry,
    scroll_x: ScrollModel,
    scroll_y: ScrollModel,
    style: FrameStyle,
    damage: DamageRegion,
) {
    let scrollbar = scrollbar_style(style);
    if let Some(rect) = geometry.scrollbars.vertical {
        paint_bar(
            console,
            rect,
            ScrollBarOrientation::Vertical,
            scroll_y,
            scrollbar,
            damage,
        );
    }
    if let Some(rect) = geometry.scrollbars.horizontal {
        paint_bar(
            console,
            rect,
            ScrollBarOrientation::Horizontal,
            scroll_x,
            scrollbar,
            damage,
        );
    }
}

/// Hit-test frame-owned scroll bars at zero-based `(x, y)`.
#[must_use]
pub fn frame_scroll_hit(
    geometry: &FrameGeometry,
    scroll_x: ScrollModel,
    scroll_y: ScrollModel,
    x: i64,
    y: i64,
) -> Option<FrameScrollHit> {
    if let Some(rect) = geometry.scrollbars.vertical
        && let Some(hit) = hit_bar(rect, ScrollBarOrientation::Vertical, scroll_y, x, y)
    {
        return Some(FrameScrollHit::Vertical(hit));
    }
    if let Some(rect) = geometry.scrollbars.horizontal
        && let Some(hit) = hit_bar(rect, ScrollBarOrientation::Horizontal, scroll_x, x, y)
    {
        return Some(FrameScrollHit::Horizontal(hit));
    }
    None
}

impl ViewRegistry {
    /// Replace logical content size, refresh geometry, and clamp scroll offsets.
    ///
    /// Returns `false` when `id` is not a scrollable frame root or geometry is invalid.
    pub fn set_frame_content_size(&mut self, id: ViewId, width: i64, height: i64) -> bool {
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        if !self
            .frame_roots
            .get(&root)
            .is_some_and(|state| state.capabilities.scrollable)
        {
            return false;
        }
        let content_size = FrameContentSize::new(width, height);
        if let Some(state) = self.frame_roots.get_mut(&root) {
            state.content_size = content_size;
        }
        self.refresh_frame_geometry(root)
    }

    /// Scroll a frame root by signed cell deltas.
    pub fn scroll_frame(&mut self, id: ViewId, delta_x: i64, delta_y: i64) -> bool {
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        let Some(state) = self.frame_roots.get_mut(&root) else {
            return false;
        };
        if !state.capabilities.scrollable {
            return false;
        }
        let changed_x = if delta_x == 0 {
            false
        } else {
            state.scroll_x.scroll_by(delta_x)
        };
        let changed_y = if delta_y == 0 {
            false
        } else {
            state.scroll_y.scroll_by(delta_y)
        };
        changed_x || changed_y
    }

    /// Set absolute scroll offsets for a frame root.
    pub fn set_frame_scroll_offset(&mut self, id: ViewId, offset_x: i64, offset_y: i64) -> bool {
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        let Some(state) = self.frame_roots.get_mut(&root) else {
            return false;
        };
        if !state.capabilities.scrollable {
            return false;
        }
        let ox = usize::try_from(offset_x.max(0)).unwrap_or(usize::MAX);
        let oy = usize::try_from(offset_y.max(0)).unwrap_or(usize::MAX);
        let changed_x = state.scroll_x.set_offset(ox);
        let changed_y = state.scroll_y.set_offset(oy);
        changed_x || changed_y
    }

    /// Return scroll offsets and logical content size for one frame root.
    #[must_use]
    pub fn frame_scroll_state(&self, id: ViewId) -> Option<FrameScrollState> {
        let state = self.frame_root_state(id)?;
        Some(FrameScrollState {
            offset_x: state.scroll_x.offset() as i64,
            offset_y: state.scroll_y.offset() as i64,
            content_width: state.content_size.width,
            content_height: state.content_size.height,
        })
    }

    /// Hit-test frame scroll chrome for one root at zero-based `(x, y)`.
    #[must_use]
    pub fn frame_scroll_hit_at(&self, id: ViewId, x: i64, y: i64) -> Option<FrameScrollHit> {
        let state = self.frame_root_state(id)?;
        if !state.capabilities.scrollable {
            return None;
        }
        frame_scroll_hit(&state.geometry, state.scroll_x, state.scroll_y, x, y)
    }

    /// Apply one scroll-bar hit on a frame root.
    pub fn apply_frame_scroll_hit(&mut self, id: ViewId, hit: FrameScrollHit) -> bool {
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        let Some(state) = self.frame_roots.get_mut(&root) else {
            return false;
        };
        match hit {
            FrameScrollHit::Vertical(bar_hit) => apply_bar_hit(&mut state.scroll_y, bar_hit),
            FrameScrollHit::Horizontal(bar_hit) => apply_bar_hit(&mut state.scroll_x, bar_hit),
        }
    }

    /// Begin a captured thumb drag on frame scroll chrome.
    pub fn begin_frame_scroll_thumb_drag(
        &mut self,
        id: ViewId,
        hit: FrameScrollHit,
        x: i64,
        y: i64,
    ) -> bool {
        let (FrameScrollHit::Vertical(ScrollBarHit::Thumb)
        | FrameScrollHit::Horizontal(ScrollBarHit::Thumb)) = hit
        else {
            return false;
        };
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        let state = match self.frame_root_state(root) {
            Some(state) => *state,
            None => return false,
        };
        let (orientation, rect, scroll) = match hit {
            FrameScrollHit::Vertical(_) => (
                ScrollBarOrientation::Vertical,
                match state.geometry.scrollbars.vertical {
                    Some(rect) => rect,
                    None => return false,
                },
                state.scroll_y,
            ),
            FrameScrollHit::Horizontal(_) => (
                ScrollBarOrientation::Horizontal,
                match state.geometry.scrollbars.horizontal {
                    Some(rect) => rect,
                    None => return false,
                },
                state.scroll_x,
            ),
        };
        let (track, track_cell) = match track_cell_at(rect, orientation, x, y) {
            Some(value) => value,
            None => return false,
        };
        let thumb = thumb_geometry(scroll, track);
        let grab = track_cell.saturating_sub(thumb.start);
        self.frame_scroll_interaction = Some(FrameScrollInteraction {
            root,
            orientation,
            grab,
        });
        self.capture_pointer(root)
    }

    /// Update scroll offset while dragging a frame scroll thumb.
    pub fn drag_frame_scroll_thumb(&mut self, x: i64, y: i64) -> bool {
        let interaction = match self.frame_scroll_interaction {
            Some(interaction) => interaction,
            None => return false,
        };
        let state = match self.frame_roots.get_mut(&interaction.root) {
            Some(state) => state,
            None => return false,
        };
        let (rect, scroll) = match interaction.orientation {
            ScrollBarOrientation::Vertical => (
                match state.geometry.scrollbars.vertical {
                    Some(rect) => rect,
                    None => return false,
                },
                &mut state.scroll_y,
            ),
            ScrollBarOrientation::Horizontal => (
                match state.geometry.scrollbars.horizontal {
                    Some(rect) => rect,
                    None => return false,
                },
                &mut state.scroll_x,
            ),
        };
        let (track, track_cell) = match track_cell_at_clamped(rect, interaction.orientation, x, y) {
            Some(value) => value,
            None => return false,
        };
        scroll.set_offset(drag_offset(*scroll, track, track_cell, interaction.grab))
    }

    /// End a captured frame scroll thumb drag.
    pub fn end_frame_scroll_interaction(&mut self) -> bool {
        let had = self.frame_scroll_interaction.is_some();
        self.frame_scroll_interaction = None;
        if self.captured_pointer().is_some() {
            self.release_pointer();
        }
        had
    }

    /// Return the active frame scroll interaction, if any.
    #[must_use]
    pub fn frame_scroll_interaction(&self) -> Option<FrameScrollInteraction> {
        self.frame_scroll_interaction
    }

    /// Apply one navigation key to the frame root containing focus.
    pub fn scroll_frame_key(&mut self, id: ViewId, key: crate::ConsoleKeyEvent) -> bool {
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        let Some(state) = self.frame_roots.get_mut(&root) else {
            return false;
        };
        if !state.capabilities.scrollable {
            return false;
        }
        match frame_scroll_key_action(&key) {
            Some((delta_x, delta_y, page_y, home, end)) => {
                let mut changed = false;
                if home {
                    changed |= state.scroll_x.set_offset(0);
                    changed |= state.scroll_y.set_offset(0);
                } else if end {
                    changed |= state.scroll_x.set_offset(usize::MAX);
                    changed |= state.scroll_y.set_offset(usize::MAX);
                } else {
                    if let Some(forward) = page_y {
                        changed |= state.scroll_y.scroll_page(forward);
                    }
                    if delta_x != 0 {
                        changed |= state.scroll_x.scroll_by(delta_x);
                    }
                    if delta_y != 0 {
                        changed |= state.scroll_y.scroll_by(delta_y);
                    }
                }
                changed
            }
            None => false,
        }
    }

    /// Measure direct-child bounds into content size when still empty.
    pub(crate) fn maybe_measure_frame_content_size(&mut self, root: ViewId) {
        let Some(state) = self.frame_roots.get(&root) else {
            return;
        };
        if !state.capabilities.scrollable
            || state.content_size.width > 0
            || state.content_size.height > 0
        {
            return;
        }
        let measured = measure_children_content_size(self, root);
        if let Some(state) = self.frame_roots.get_mut(&root) {
            state.content_size = measured;
        }
    }
}

pub(crate) fn sync_frame_scroll_extents(state: &mut FrameRootState) {
    let view_w = state.geometry.view.width.max(0) as usize;
    let view_h = state.geometry.view.height.max(0) as usize;
    let content_w = state.content_size.width.max(0) as usize;
    let content_h = state.content_size.height.max(0) as usize;
    state.scroll_x.set_extents(content_w, view_w);
    state.scroll_y.set_extents(content_h, view_h);
}

fn frame_scroll_key_action(
    key: &crate::ConsoleKeyEvent,
) -> Option<(i64, i64, Option<bool>, bool, bool)> {
    use crate::key_kind_index;
    match key.kind {
        k if k == key_kind_index("Up") => Some((0, -1, None, false, false)),
        k if k == key_kind_index("Down") => Some((0, 1, None, false, false)),
        k if k == key_kind_index("Left") => Some((-1, 0, None, false, false)),
        k if k == key_kind_index("Right") => Some((1, 0, None, false, false)),
        k if k == key_kind_index("PageUp") => Some((0, 0, Some(false), false, false)),
        k if k == key_kind_index("PageDown") => Some((0, 0, Some(true), false, false)),
        k if k == key_kind_index("Home") => Some((0, 0, None, true, false)),
        k if k == key_kind_index("End") => Some((0, 0, None, false, true)),
        _ => None,
    }
}

fn measure_children_content_size(registry: &ViewRegistry, root: ViewId) -> FrameContentSize {
    let mut width = 0i64;
    let mut height = 0i64;
    for child in registry.children(root) {
        if let Some(local) = registry.local_rect(*child) {
            let right = local.x.saturating_add(local.width);
            let bottom = local.y.saturating_add(local.height);
            width = width.max(right);
            height = height.max(bottom);
        }
    }
    FrameContentSize::new(width, height)
}

fn apply_bar_hit(scroll: &mut ScrollModel, hit: ScrollBarHit) -> bool {
    match hit {
        ScrollBarHit::DecrementArrow => scroll.scroll_by(-1),
        ScrollBarHit::IncrementArrow => scroll.scroll_by(1),
        ScrollBarHit::TrackBefore => scroll.scroll_page(false),
        ScrollBarHit::TrackAfter => scroll.scroll_page(true),
        ScrollBarHit::Thumb => false,
    }
}

fn scrollbar_style(style: FrameStyle) -> ScrollBarStyle {
    ScrollBarStyle {
        bg: style.client_bg,
        fg: style.client_fg,
        thumb_fg: style.active_fg,
        arrow_fg: style.client_fg,
    }
}

fn paint_bar(
    console: &mut Console,
    rect: ViewRect,
    orientation: ScrollBarOrientation,
    scroll: ScrollModel,
    style: ScrollBarStyle,
    damage: DamageRegion,
) {
    let Some(clip) = clip_to_damage(rect, damage) else {
        return;
    };
    console.fill_rect_crt(clip, style.fg, style.bg, ' ');
    let bar_cells = match orientation {
        ScrollBarOrientation::Vertical => rect.height.max(0) as usize,
        ScrollBarOrientation::Horizontal => rect.width.max(0) as usize,
    };
    if bar_cells < 3 {
        return;
    }
    let track = track_cells(bar_cells);
    let thumb = thumb_geometry(scroll, track);
    match orientation {
        ScrollBarOrientation::Vertical => paint_vertical(console, rect, clip, thumb, style),
        ScrollBarOrientation::Horizontal => paint_horizontal(console, rect, clip, thumb, style),
    }
}

fn paint_vertical(
    console: &mut Console,
    rect: ViewRect,
    clip: ViewRect,
    thumb: ScrollBarThumb,
    style: ScrollBarStyle,
) {
    let height = rect.height.max(0) as usize;
    paint_cell(console, rect, clip, 0, '▲', style.arrow_fg, style.bg);
    paint_cell(
        console,
        rect,
        clip,
        height - 1,
        '▼',
        style.arrow_fg,
        style.bg,
    );
    for row in 1..height.saturating_sub(1) {
        let track_row = row - 1;
        let ch = if track_row >= thumb.start && track_row < thumb.start + thumb.size {
            '█'
        } else {
            '░'
        };
        let fg = if ch == '█' {
            style.thumb_fg
        } else {
            style.fg
        };
        paint_cell(console, rect, clip, row, ch, fg, style.bg);
    }
}

fn paint_horizontal(
    console: &mut Console,
    rect: ViewRect,
    clip: ViewRect,
    thumb: ScrollBarThumb,
    style: ScrollBarStyle,
) {
    let width = rect.width.max(0) as usize;
    paint_cell_at(
        console,
        rect,
        clip,
        0,
        rect.y,
        '◄',
        style.arrow_fg,
        style.bg,
    );
    paint_cell_at(
        console,
        rect,
        clip,
        width.saturating_sub(1),
        rect.y,
        '►',
        style.arrow_fg,
        style.bg,
    );
    for col in 1..width.saturating_sub(1) {
        let track_col = col - 1;
        let ch = if track_col >= thumb.start && track_col < thumb.start + thumb.size {
            '█'
        } else {
            '░'
        };
        let fg = if ch == '█' {
            style.thumb_fg
        } else {
            style.fg
        };
        paint_cell_at(console, rect, clip, col, rect.y, ch, fg, style.bg);
    }
}

fn hit_bar(
    rect: ViewRect,
    orientation: ScrollBarOrientation,
    scroll: ScrollModel,
    x: i64,
    y: i64,
) -> Option<ScrollBarHit> {
    if !rect.contains_point(x, y) {
        return None;
    }
    let bar_cells = match orientation {
        ScrollBarOrientation::Vertical => rect.height.max(0) as usize,
        ScrollBarOrientation::Horizontal => rect.width.max(0) as usize,
    };
    let cell = match orientation {
        ScrollBarOrientation::Vertical => y.saturating_sub(rect.y) as usize,
        ScrollBarOrientation::Horizontal => x.saturating_sub(rect.x) as usize,
    };
    hit_zone(scroll, bar_cells, cell)
}

fn track_cell_at(
    rect: ViewRect,
    orientation: ScrollBarOrientation,
    x: i64,
    y: i64,
) -> Option<(usize, usize)> {
    let (bar_cells, cell) = bar_cell(rect, orientation, x, y)?;
    if cell == 0 || cell + 1 == bar_cells {
        return None;
    }
    Some((track_cells(bar_cells), cell - 1))
}

fn track_cell_at_clamped(
    rect: ViewRect,
    orientation: ScrollBarOrientation,
    x: i64,
    y: i64,
) -> Option<(usize, usize)> {
    let (bar_cells, cell) = bar_cell(rect, orientation, x, y)?;
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

fn bar_cell(
    rect: ViewRect,
    orientation: ScrollBarOrientation,
    x: i64,
    y: i64,
) -> Option<(usize, usize)> {
    if !rect.contains_point(x, y) {
        return None;
    }
    let bar_cells = match orientation {
        ScrollBarOrientation::Vertical => rect.height.max(0) as usize,
        ScrollBarOrientation::Horizontal => rect.width.max(0) as usize,
    };
    if bar_cells < 3 {
        return None;
    }
    let cell = match orientation {
        ScrollBarOrientation::Vertical => y.saturating_sub(rect.y) as usize,
        ScrollBarOrientation::Horizontal => x.saturating_sub(rect.x) as usize,
    };
    (cell < bar_cells).then_some((bar_cells, cell))
}

fn paint_cell(
    console: &mut Console,
    rect: ViewRect,
    clip: ViewRect,
    row: usize,
    ch: char,
    fg: u8,
    bg: u8,
) {
    paint_cell_at(console, rect, clip, 0, rect.y + row as i64, ch, fg, bg);
}

fn paint_cell_at(
    console: &mut Console,
    rect: ViewRect,
    clip: ViewRect,
    col: usize,
    y: i64,
    ch: char,
    fg: u8,
    bg: u8,
) {
    let x = rect.x + col as i64;
    if clip.contains_point(x, y) {
        console.write_char_at_crt(x, y, ch, fg, bg);
    }
}

fn clip_to_damage(rect: ViewRect, damage: DamageRegion) -> Option<ViewRect> {
    match damage {
        DamageRegion::FullFrame => Some(rect),
        DamageRegion::Rect(dirty) => rect.intersection(dirty),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widget::frame::{FrameCapabilities, FrameKind, FrameRootSpec};

    fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn frame_scroll_clamps_offset_to_content_minus_viewport() {
        let mut registry = ViewRegistry::default();
        let frame = registry
            .register_frame_root(FrameRootSpec {
                kind: FrameKind::Window,
                outer: rect(0, 0, 10, 8),
                content_size: FrameContentSize::new(20, 12),
                capabilities: FrameCapabilities::scrollable(),
                options: Default::default(),
            })
            .expect("frame");
        assert!(registry.set_frame_scroll_offset(frame.view_id, 100, 100));
        let state = registry.frame_scroll_state(frame.view_id).unwrap();
        assert_eq!(state.offset_x, 13);
        assert_eq!(state.offset_y, 7);
    }

    #[test]
    fn frame_scroll_hit_detects_vertical_arrow_cells() {
        let geometry = FrameGeometry::resolve(
            rect(0, 0, 10, 8),
            FrameContentSize::new(5, 20),
            FrameCapabilities::scrollable(),
        )
        .expect("geometry");
        let scroll_y = ScrollModel::new(20, 6);
        assert_eq!(
            frame_scroll_hit(&geometry, ScrollModel::new(0, 0), scroll_y, 8, 1),
            Some(FrameScrollHit::Vertical(ScrollBarHit::DecrementArrow))
        );
    }
}
