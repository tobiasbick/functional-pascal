use crate::{Console, DamageRegion, ViewRect};

use super::{clip_rect_to_damage, paint_chars};
use crate::text::{char_display_offset, layout_display_cells};

/// CRT colors used while painting a single-line text input control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputLineStyle {
    /// Normal input background.
    pub bg: u8,
    /// Normal input foreground.
    pub fg: u8,
    /// Cursor cell background while focused.
    pub cursor_bg: u8,
    /// Cursor cell foreground while focused.
    pub cursor_fg: u8,
    /// Foreground used while disabled.
    pub disabled_fg: u8,
}

impl Default for InputLineStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 0,
            cursor_bg: 0,
            cursor_fg: 15,
            disabled_fg: 8,
        }
    }
}

/// Single-line editable text control for dialogs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputLineWidget {
    text: String,
    cursor: usize,
    scroll_offset: usize,
    /// Whether the input accepts editing and paints with normal colors.
    pub enabled: bool,
    /// Whether the input paints a visible cursor.
    pub focused: bool,
    /// CRT style used for painting.
    pub style: InputLineStyle,
}

impl InputLineWidget {
    /// Create an enabled input line with the cursor at the end of `text`.
    #[must_use]
    pub fn new(text: impl Into<String>, style: InputLineStyle) -> Self {
        let text = text.into();
        let cursor = text.chars().count();
        Self {
            text,
            cursor,
            scroll_offset: 0,
            enabled: true,
            focused: false,
            style,
        }
    }

    /// Return the current text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the cursor position as a character index.
    #[must_use]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Return the requested horizontal scroll offset as a character index.
    #[must_use]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Replace the text and clamp the cursor to the new end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.cursor.min(self.text_len());
        self.scroll_offset = self.scroll_offset.min(self.cursor);
    }

    /// Set the cursor position, clamped to the valid text range.
    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.text_len());
        self.scroll_offset = self.scroll_offset.min(self.cursor);
    }

    /// Move the cursor left by one character.
    pub fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.scroll_offset = self.scroll_offset.min(self.cursor);
    }

    /// Move the cursor right by one character.
    pub fn move_cursor_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.text_len());
    }

    /// Insert one character at the cursor.
    pub fn insert_char(&mut self, ch: char) {
        let byte_index = self.byte_index(self.cursor);
        self.text.insert(byte_index, ch);
        self.cursor += 1;
    }

    /// Insert text at the cursor; this is the retained-side paste primitive.
    pub fn insert_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let byte_index = self.byte_index(self.cursor);
        self.text.insert_str(byte_index, text);
        self.cursor += text.chars().count();
    }

    /// Delete the character before the cursor.
    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        self.remove_char_at(self.cursor)
    }

    /// Delete the character at the cursor.
    pub fn delete(&mut self) -> bool {
        self.remove_char_at(self.cursor)
    }

    /// Paint the input line into `rect`, clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = clip_rect_to_damage(rect, damage) else {
            return;
        };
        let fg = self.foreground_color();
        console.fill_rect_crt(clip, fg, self.style.bg, ' ');

        let view_width = rect.width.max(0) as usize;
        if view_width == 0 {
            return;
        }

        let scroll = self.effective_scroll(view_width);
        let display_y = rect.y + (rect.height.saturating_sub(1) / 2).max(0);
        let text_rect = ViewRect {
            x: rect.x,
            y: display_y,
            width: rect.width,
            height: 1,
        };
        let visible = &self.text[self.byte_index(scroll)..];
        paint_chars(
            console,
            text_rect,
            clip,
            layout_display_cells(visible, view_width).into_iter(),
            |_| fg,
            self.style.bg,
        );

        if self.focused && self.enabled {
            self.paint_cursor(console, text_rect, clip, scroll);
        }
    }

    fn foreground_color(&self) -> u8 {
        if self.enabled {
            self.style.fg
        } else {
            self.style.disabled_fg
        }
    }

    fn effective_scroll(&self, view_width: usize) -> usize {
        if self.cursor < self.scroll_offset {
            self.cursor
        } else if self.cursor >= self.scroll_offset.saturating_add(view_width) {
            self.cursor.saturating_sub(view_width.saturating_sub(1))
        } else {
            self.scroll_offset
        }
    }

    fn paint_cursor(&self, console: &mut Console, rect: ViewRect, clip: ViewRect, scroll: usize) {
        let cursor_col = char_display_offset(&self.text, self.cursor)
            .saturating_sub(char_display_offset(&self.text, scroll));
        if cursor_col >= rect.width.max(0) as usize {
            return;
        }
        let x = rect.x + cursor_col as i64;
        if !clip.contains_point(x, rect.y) {
            return;
        }
        let ch = self.text.chars().nth(self.cursor).unwrap_or(' ');
        console.write_char_at_crt(x, rect.y, ch, self.style.cursor_fg, self.style.cursor_bg);
    }

    fn text_len(&self) -> usize {
        self.text.chars().count()
    }

    fn byte_index(&self, char_index: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_index)
            .map_or(self.text.len(), |(byte_index, _)| byte_index)
    }

    fn remove_char_at(&mut self, char_index: usize) -> bool {
        if char_index >= self.text_len() {
            return false;
        }
        let start = self.byte_index(char_index);
        let end = self.byte_index(char_index + 1);
        self.text.replace_range(start..end, "");
        self.scroll_offset = self.scroll_offset.min(self.cursor);
        true
    }
}
