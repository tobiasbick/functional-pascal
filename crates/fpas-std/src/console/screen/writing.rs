use super::{ConsoleState, ScreenCell};

impl ConsoleState {
    pub(in super::super) fn clear_window(&mut self) {
        let blank = self.blank_cell();
        for y in self.window.top..=self.window.bottom {
            for x in self.window.left..=self.window.right {
                let idx = self.index(x, y);
                self.cells[idx] = blank;
            }
        }
        self.cursor_x = 1;
        self.cursor_y = 1;
        self.pending_wrap = false;
    }

    pub(in super::super) fn clear_eol(&mut self) {
        let blank = self.blank_cell();
        let y = self.abs_y();
        for x in self.abs_x()..=self.window.right {
            let idx = self.index(x, y);
            self.cells[idx] = blank;
        }
    }

    pub(in super::super) fn del_line(&mut self) {
        let abs_y = self.abs_y();
        for y in abs_y..self.window.bottom {
            for x in self.window.left..=self.window.right {
                let dst = self.index(x, y);
                let src = self.index(x, y + 1);
                self.cells[dst] = self.cells[src];
            }
        }
        let blank = self.blank_cell();
        for x in self.window.left..=self.window.right {
            let idx = self.index(x, self.window.bottom);
            self.cells[idx] = blank;
        }
    }

    pub(in super::super) fn ins_line(&mut self) {
        let abs_y = self.abs_y();
        for y in (abs_y + 1..=self.window.bottom).rev() {
            for x in self.window.left..=self.window.right {
                let dst = self.index(x, y);
                let src = self.index(x, y - 1);
                self.cells[dst] = self.cells[src];
            }
        }
        let blank = self.blank_cell();
        for x in self.window.left..=self.window.right {
            let idx = self.index(x, abs_y);
            self.cells[idx] = blank;
        }
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
                let x = self.abs_x();
                let y = self.abs_y();
                let idx = self.index(x, y);
                self.cells[idx] = ScreenCell {
                    ch,
                    fg: self.active_fg,
                    bg: self.active_bg,
                };
                if self.cursor_x == self.window_width() {
                    self.pending_wrap = true;
                } else {
                    self.cursor_x += 1;
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
        for y in self.window.top..self.window.bottom {
            for x in self.window.left..=self.window.right {
                let dst = self.index(x, y);
                let src = self.index(x, y + 1);
                self.cells[dst] = self.cells[src];
            }
        }
        let blank = self.blank_cell();
        for x in self.window.left..=self.window.right {
            let idx = self.index(x, self.window.bottom);
            self.cells[idx] = blank;
        }
    }
}
