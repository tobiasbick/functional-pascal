//! Multi-line editable memo control with selection, paste, and vertical scroll.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use super::paint_chars;
use super::scroll_bar::{ScrollBarStyle, ScrollBarWidget};
use crate::text::{char_display_offset, display_width, layout_display_cells};
use crate::{Console, DamageRegion, ScrollBarHit, ScrollBarOrientation, ScrollModel, ViewRect};

const EMPTY_PLACEHOLDER: &str = "(empty)";

/// Cursor position as a zero-based line and character index within that line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TextPos {
    line: usize,
    column: usize,
}

/// CRT colors used while painting a memo control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoStyle {
    /// Normal background.
    pub bg: u8,
    /// Normal foreground.
    pub fg: u8,
    /// Selected text background.
    pub selected_bg: u8,
    /// Selected text foreground.
    pub selected_fg: u8,
    /// Cursor cell background while focused.
    pub cursor_bg: u8,
    /// Cursor cell foreground while focused.
    pub cursor_fg: u8,
    /// Foreground used while disabled.
    pub disabled_fg: u8,
    /// Integrated scroll-bar style.
    pub scrollbar: ScrollBarStyle,
}

impl Default for MemoStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 0,
            selected_bg: 0,
            selected_fg: 15,
            cursor_bg: 0,
            cursor_fg: 15,
            disabled_fg: 8,
            scrollbar: ScrollBarStyle::default(),
        }
    }
}

/// Multi-line editable text control with optional selection and vertical scrolling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoWidget {
    lines: Vec<String>,
    cursor: TextPos,
    selection_anchor: Option<TextPos>,
    scroll: ScrollModel,
    thumb_drag_grab: Option<usize>,
    pub enabled: bool,
    pub focused: bool,
    pub style: MemoStyle,
}

impl MemoWidget {
    /// Create a memo from multiline `text` sized for `viewport_lines` visible rows.
    #[must_use]
    pub fn new(text: impl Into<String>, viewport_lines: usize) -> Self {
        let lines = lines_from_text(&text.into());
        let cursor = TextPos {
            line: lines.len().saturating_sub(1),
            column: lines.last().map_or(0, |line| line.chars().count()),
        };
        Self {
            scroll: ScrollModel::new(lines.len(), viewport_lines),
            lines,
            cursor,
            selection_anchor: None,
            thumb_drag_grab: None,
            enabled: true,
            focused: false,
            style: MemoStyle::default(),
        }
    }

    /// Return the memo text joined with `\n`.
    #[must_use]
    pub fn text(&self) -> String {
        text_from_lines(&self.lines)
    }

    /// Zero-based cursor line index.
    #[must_use]
    pub fn cursor_line(&self) -> usize {
        self.cursor.line
    }

    /// Zero-based cursor column as a character index within the cursor line.
    #[must_use]
    pub fn cursor_column(&self) -> usize {
        self.cursor.column
    }

    /// Vertical scroll offset in lines.
    #[must_use]
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Selection anchor line, or `None` when no range is active.
    #[must_use]
    pub fn selection_anchor_line(&self) -> Option<usize> {
        self.selection_anchor.map(|pos| pos.line)
    }

    /// Selection anchor column, or `None` when no range is active.
    #[must_use]
    pub fn selection_anchor_column(&self) -> Option<usize> {
        self.selection_anchor.map(|pos| pos.column)
    }

    /// Return whether a thumb drag is active on the integrated scroll bar.
    #[must_use]
    pub fn thumb_drag_active(&self) -> bool {
        self.thumb_drag_grab.is_some()
    }

    /// Replace memo text and reset cursor, selection, and scroll.
    pub fn set_text(&mut self, text: impl Into<String>, viewport_lines: usize) {
        let lines = lines_from_text(&text.into());
        self.lines = lines;
        self.cursor = TextPos { line: 0, column: 0 };
        self.selection_anchor = None;
        self.scroll = ScrollModel::new(self.lines.len(), viewport_lines);
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

    /// Move the cursor with optional selection extension.
    pub fn move_cursor(&mut self, delta_line: i64, delta_column: i64, extend_selection: bool) {
        if !extend_selection {
            self.selection_anchor = None;
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        let line = self.cursor.line as i64 + delta_line;
        let line = line.clamp(0, self.lines.len().saturating_sub(1) as i64) as usize;
        let line_len = self.lines.get(line).map_or(0, |text| text.chars().count());
        let column = (self.cursor.column as i64 + delta_column).clamp(0, line_len as i64) as usize;
        self.cursor = TextPos { line, column };
        self.ensure_cursor_visible();
    }

    /// Set the cursor from a click in view coordinates.
    pub fn set_cursor_from_click(&mut self, content: ViewRect, mouse_x: i64, mouse_y: i64) {
        let row = mouse_y.saturating_sub(content.y).max(0) as usize;
        let line = self
            .scroll
            .offset()
            .saturating_add(row)
            .min(self.lines.len().saturating_sub(1));
        let col = mouse_x.saturating_sub(content.x).max(0) as usize;
        let column =
            char_index_at_display_col(self.lines.get(line).map_or("", String::as_str), col);
        self.cursor = TextPos { line, column };
        self.selection_anchor = None;
        self.ensure_cursor_visible();
    }

    /// Move the cursor to the start or end of the current line.
    pub fn move_cursor_line_edge(&mut self, to_end: bool, extend_selection: bool) {
        if !extend_selection {
            self.selection_anchor = None;
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
        let line_len = self
            .lines
            .get(self.cursor.line)
            .map_or(0, |line| line.chars().count());
        self.cursor.column = if to_end { line_len } else { 0 };
        self.ensure_cursor_visible();
    }

    /// Insert one character, replacing any active selection.
    pub fn insert_char(&mut self, ch: char) {
        self.delete_selection();
        if ch == '\n' {
            self.split_line();
            return;
        }
        let line_index = self.cursor.line;
        let column = self.cursor.column;
        let line = self.line_mut(line_index);
        let byte = byte_index(line, column);
        line.insert(byte, ch);
        self.cursor.column += 1;
        self.selection_anchor = None;
        self.sync_scroll_extents();
    }

    /// Insert text at the cursor, supporting embedded newlines.
    pub fn insert_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.delete_selection();
        for ch in text.chars() {
            self.insert_char(ch);
        }
    }

    /// Delete the character before the cursor or the active selection.
    pub fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
            return self.remove_char_at_cursor();
        }
        if self.cursor.line == 0 {
            return false;
        }
        let current = self.lines.remove(self.cursor.line);
        self.cursor.line -= 1;
        self.cursor.column = self.lines[self.cursor.line].chars().count();
        self.lines[self.cursor.line].push_str(&current);
        self.selection_anchor = None;
        self.sync_scroll_extents();
        true
    }

    /// Delete the character at the cursor or the active selection.
    pub fn delete(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        self.remove_char_at_cursor()
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

    /// Paint memo content, selection, cursor, and integrated scroll bar.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = damage.clip_rect(rect) else {
            return;
        };
        let fg = if self.enabled {
            self.style.fg
        } else {
            self.style.disabled_fg
        };
        let content = self.content_rect(rect);
        console.fill_rect_crt(content, fg, self.style.bg, ' ');

        if self.is_empty() && !self.focused && content.height > 0 {
            let line_rect = ViewRect {
                x: content.x,
                y: content.y,
                width: content.width,
                height: 1,
            };
            paint_chars(
                console,
                line_rect,
                clip,
                layout_display_cells(EMPTY_PLACEHOLDER, line_rect.width.max(0) as usize)
                    .into_iter(),
                |_| self.style.disabled_fg,
                self.style.bg,
            );
        }

        let (sel_start, sel_end) = self.normalized_selection();
        for (row, line) in self
            .lines
            .iter()
            .enumerate()
            .skip(self.scroll.offset())
            .take(content.height.max(0) as usize)
        {
            let line_index = row + self.scroll.offset();
            let line_rect = ViewRect {
                x: content.x,
                y: content.y + row as i64,
                width: content.width,
                height: 1,
            };
            for (offset, ch) in layout_display_cells(line, line_rect.width.max(0) as usize) {
                let column = char_index_at_display_col(line, offset);
                let selected = pos_in_range(
                    TextPos {
                        line: line_index,
                        column,
                    },
                    sel_start,
                    sel_end,
                );
                let (paint_fg, paint_bg) = if selected {
                    (self.style.selected_fg, self.style.selected_bg)
                } else {
                    (fg, self.style.bg)
                };
                paint_chars(
                    console,
                    line_rect,
                    clip,
                    std::iter::once((offset, ch)),
                    |_| paint_fg,
                    paint_bg,
                );
            }
        }

        if self.focused && self.enabled && self.cursor.line >= self.scroll.offset() {
            let cursor_row = self.cursor.line.saturating_sub(self.scroll.offset());
            if cursor_row < content.height.max(0) as usize {
                let x = content.x
                    + char_display_offset(
                        self.lines.get(self.cursor.line).map_or("", String::as_str),
                        self.cursor.column,
                    ) as i64;
                let y = content.y + cursor_row as i64;
                if clip.contains_point(x, y) {
                    let ch = self
                        .lines
                        .get(self.cursor.line)
                        .and_then(|line| line.chars().nth(self.cursor.column))
                        .unwrap_or(' ');
                    console.write_char_at_crt(x, y, ch, self.style.cursor_fg, self.style.cursor_bg);
                }
            }
        }

        if let Some(bar_rect) = self.scrollbar_rect(rect) {
            let mut bar = ScrollBarWidget::with_scroll(ScrollBarOrientation::Vertical, self.scroll);
            bar.style = self.style.scrollbar;
            bar.paint(console, bar_rect, damage);
        }
    }

    fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    fn normalized_selection(&self) -> (Option<TextPos>, Option<TextPos>) {
        let Some(anchor) = self.selection_anchor else {
            return (None, None);
        };
        if anchor == self.cursor {
            return (None, None);
        }
        if anchor.line < self.cursor.line
            || (anchor.line == self.cursor.line && anchor.column <= self.cursor.column)
        {
            (Some(anchor), Some(self.cursor))
        } else {
            (Some(self.cursor), Some(anchor))
        }
    }

    fn delete_selection(&mut self) -> bool {
        let (Some(start), Some(end)) = self.normalized_selection() else {
            return false;
        };
        if start == end {
            return false;
        }
        if start.line == end.line {
            let line = self.line_mut(start.line);
            let start_byte = byte_index(line, start.column);
            let end_byte = byte_index(line, end.column);
            line.replace_range(start_byte..end_byte, "");
            self.cursor = start;
        } else {
            let first_tail = {
                let line = self.line_mut(start.line);
                let start_byte = byte_index(line, start.column);
                line[start_byte..].to_string()
            };
            let last_head = {
                let line = self.line_mut(end.line);
                let end_byte = byte_index(line, end.column);
                line[..end_byte].to_string()
            };
            self.lines[start.line] = format!("{last_head}{first_tail}");
            self.lines.drain(start.line + 1..=end.line);
            self.cursor = TextPos {
                line: start.line,
                column: last_head.chars().count(),
            };
        }
        self.selection_anchor = None;
        self.sync_scroll_extents();
        true
    }

    fn split_line(&mut self) {
        let line_index = self.cursor.line;
        let column = self.cursor.column;
        let tail = {
            let line = self.line_mut(line_index);
            let byte = byte_index(line, column);
            line.split_off(byte)
        };
        self.lines.insert(line_index + 1, tail);
        self.cursor.line += 1;
        self.cursor.column = 0;
        self.sync_scroll_extents();
    }

    fn remove_char_at_cursor(&mut self) -> bool {
        let line_index = self.cursor.line;
        let column = self.cursor.column;
        let line_len = self
            .lines
            .get(line_index)
            .map_or(0, |line| line.chars().count());
        if column < line_len {
            let line = self.line_mut(line_index);
            let start = byte_index(line, column);
            let end = byte_index(line, column + 1);
            line.replace_range(start..end, "");
            self.sync_scroll_extents();
            return true;
        }
        if line_index + 1 >= self.lines.len() {
            return false;
        }
        let next = self.lines.remove(line_index + 1);
        self.lines[line_index].push_str(&next);
        self.sync_scroll_extents();
        true
    }

    fn line_mut(&mut self, line: usize) -> &mut String {
        &mut self.lines[line]
    }

    fn ensure_cursor_visible(&mut self) {
        if self.cursor.line < self.scroll.offset() {
            let _ = self.scroll.set_offset(self.cursor.line);
        } else if self.cursor.line >= self.scroll.offset() + self.scroll.viewport_len() {
            let _ = self.scroll.set_offset(
                self.cursor
                    .line
                    .saturating_sub(self.scroll.viewport_len().saturating_sub(1)),
            );
        }
    }

    fn sync_scroll_extents(&mut self) {
        self.scroll
            .set_extents(self.lines.len(), self.scroll.viewport_len());
        self.ensure_cursor_visible();
    }
}

fn lines_from_text(text: &str) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    text.split('\n').map(str::to_string).collect()
}

fn text_from_lines(lines: &[String]) -> String {
    lines.join("\n")
}

fn byte_index(line: &str, char_index: usize) -> usize {
    line.char_indices()
        .nth(char_index)
        .map_or(line.len(), |(byte, _)| byte)
}

fn char_index_at_display_col(line: &str, col: usize) -> usize {
    let mut display = 0usize;
    for (index, ch) in line.chars().enumerate() {
        if display >= col {
            return index;
        }
        display += usize::from(display_width(ch));
    }
    line.chars().count()
}

fn pos_in_range(pos: TextPos, start: Option<TextPos>, end: Option<TextPos>) -> bool {
    let (Some(start), Some(end)) = (start, end) else {
        return false;
    };
    if pos.line < start.line || pos.line > end.line {
        return false;
    }
    if pos.line == start.line && pos.column < start.column {
        return false;
    }
    if pos.line == end.line && pos.column >= end.column {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memo_inserts_embedded_newlines() {
        let mut memo = MemoWidget::new("", 2);
        memo.insert_str("x\ny");
        assert_eq!(memo.text(), "x\ny");
    }

    #[test]
    fn memo_deletes_selection_range() {
        let mut memo = MemoWidget::new("abcdef", 1);
        memo.selection_anchor = Some(TextPos { line: 0, column: 1 });
        memo.cursor = TextPos { line: 0, column: 4 };
        assert!(memo.delete_selection());
        assert_eq!(memo.text(), "aef");
        assert_eq!(memo.cursor_column(), 1);
    }

    #[test]
    fn memo_paste_replaces_selection() {
        let mut memo = MemoWidget::new("hello", 1);
        memo.selection_anchor = Some(TextPos { line: 0, column: 1 });
        memo.cursor = TextPos { line: 0, column: 4 };
        memo.insert_str("p");
        assert_eq!(memo.text(), "hpo");
    }
}
