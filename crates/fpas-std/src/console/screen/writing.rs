use super::ConsoleState;
use crate::console::cell::ConsoleCell;
use crate::text::cell_width::display_width;

impl ConsoleState {
    pub(in super::super) fn clear_window(&mut self) {
        let Some(dirty) = self.clip_window(self.window) else {
            return;
        };
        let blank = self.blank_cell();
        for y in dirty.top..=dirty.bottom {
            for x in dirty.left..=dirty.right {
                let idx = self.index(x, y);
                self.cells[idx] = blank.clone();
            }
        }
        self.cursor_x = 1;
        self.cursor_y = 1;
        self.pending_wrap = false;
        self.normalize_wide_cells();
        self.mark_damage_rect(dirty);
    }

    pub(in super::super) fn clear_eol(&mut self) {
        let blank = self.blank_cell();
        let y = self.abs_y();
        let left = self.abs_x();
        let Some(dirty) = self.clip_window(super::WindowRect {
            left,
            top: y,
            right: self.window.right,
            bottom: y,
        }) else {
            return;
        };
        for x in dirty.left..=dirty.right {
            let idx = self.index(x, y);
            self.cells[idx] = blank.clone();
        }
        self.normalize_wide_cells();
        self.mark_damage_rect(dirty);
    }

    pub(in super::super) fn del_line(&mut self) {
        let abs_y = self.abs_y();
        let Some(dirty) = self.clip_window(super::WindowRect {
            left: self.window.left,
            top: abs_y,
            right: self.window.right,
            bottom: self.window.bottom,
        }) else {
            return;
        };
        for y in dirty.top..dirty.bottom {
            for x in dirty.left..=dirty.right {
                let dst = self.index(x, y);
                let src = self.index(x, y + 1);
                self.cells[dst] = self.cells[src].clone();
            }
        }
        let blank = self.blank_cell();
        for x in dirty.left..=dirty.right {
            let idx = self.index(x, dirty.bottom);
            self.cells[idx] = blank.clone();
        }
        self.normalize_wide_cells();
        self.mark_damage_rect(dirty);
    }

    pub(in super::super) fn ins_line(&mut self) {
        let abs_y = self.abs_y();
        let Some(dirty) = self.clip_window(super::WindowRect {
            left: self.window.left,
            top: abs_y,
            right: self.window.right,
            bottom: self.window.bottom,
        }) else {
            return;
        };
        for y in (dirty.top + 1..=dirty.bottom).rev() {
            for x in dirty.left..=dirty.right {
                let dst = self.index(x, y);
                let src = self.index(x, y - 1);
                self.cells[dst] = self.cells[src].clone();
            }
        }
        let blank = self.blank_cell();
        for x in dirty.left..=dirty.right {
            let idx = self.index(x, dirty.top);
            self.cells[idx] = blank.clone();
        }
        self.normalize_wide_cells();
        self.mark_damage_rect(dirty);
    }

    pub(in super::super) fn write_text(&mut self, s: &str, newline: bool) {
        for ch in s.chars() {
            self.write_char(ch);
        }
        if newline {
            self.new_line();
        }
    }

    fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.pending_wrap = false;
                self.new_line();
            }
            '\r' => {
                self.pending_wrap = false;
                self.cursor_x = 1;
            }
            _ => {
                if self.pending_wrap {
                    self.pending_wrap = false;
                    self.new_line();
                }
                let width = u16::from(display_width(ch));
                if width == 0 {
                    return;
                }
                if width == 2 && self.cursor_x == self.window_width() {
                    self.new_line();
                }
                let x = self.abs_x();
                let y = self.abs_y();
                if self.can_paint_cell(x, y) {
                    self.put_cell(
                        x,
                        y,
                        ConsoleCell {
                            glyph: ch.to_string(),
                            foreground: self.active_fg.into(),
                            background: self.active_bg.into(),
                        },
                    );
                }
                if self.cursor_x.saturating_add(width - 1) == self.window_width() {
                    self.pending_wrap = true;
                } else {
                    self.cursor_x += width;
                }
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_x = 1;
        self.pending_wrap = false;
        if self.cursor_y == self.window_height() {
            self.scroll_window_up();
        } else {
            self.cursor_y += 1;
        }
    }

    fn scroll_window_up(&mut self) {
        let Some(dirty) = self.clip_window(self.window) else {
            return;
        };
        for y in dirty.top..dirty.bottom {
            for x in dirty.left..=dirty.right {
                let dst = self.index(x, y);
                let src = self.index(x, y + 1);
                self.cells[dst] = self.cells[src].clone();
            }
        }
        let blank = self.blank_cell();
        for x in dirty.left..=dirty.right {
            let idx = self.index(x, dirty.bottom);
            self.cells[idx] = blank.clone();
        }
        self.normalize_wide_cells();
        self.mark_damage_rect(dirty);
    }
}
