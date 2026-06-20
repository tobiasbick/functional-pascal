use super::{ConsoleState, RenderColor, ScreenCell, WindowRect};

impl ConsoleState {
    /// Write `text` at zero-based terminal coordinates using CRT color indices.
    pub(in super::super) fn write_text_at_crt(
        &mut self,
        x: i64,
        y: i64,
        text: &str,
        fg: u8,
        bg: u8,
    ) {
        if text.is_empty() {
            return;
        }

        let start_x = x.saturating_add(1);
        let start_y = y.saturating_add(1);
        if start_x > i64::from(self.width) || start_y > i64::from(self.height) {
            return;
        }

        let mut damage: Option<WindowRect> = None;
        let mut col = start_x;
        for ch in text.chars() {
            if col > i64::from(self.width) || start_y > i64::from(self.height) {
                break;
            }
            if col < 1 || start_y < 1 || !self.can_paint_cell(col as u16, start_y as u16) {
                col += 1;
                continue;
            }
            let idx = self.index(col as u16, start_y as u16);
            self.cells[idx] = ScreenCell {
                ch,
                fg: RenderColor::Crt(fg),
                bg: RenderColor::Crt(bg),
            };
            let cell = WindowRect {
                left: col as u16,
                top: start_y as u16,
                right: col as u16,
                bottom: start_y as u16,
            };
            damage = Some(match damage {
                Some(existing) => existing.union(cell),
                None => cell,
            });
            col += 1;
        }

        if let Some(window) = damage {
            self.mark_damage_rect(window);
        }
    }

    /// Write one character at zero-based terminal coordinates using CRT color indices.
    pub(in super::super) fn write_char_at_crt(&mut self, x: i64, y: i64, ch: char, fg: u8, bg: u8) {
        let start_x = x.saturating_add(1);
        let start_y = y.saturating_add(1);
        if start_x > i64::from(self.width) || start_y > i64::from(self.height) {
            return;
        }
        if start_x < 1 || start_y < 1 || !self.can_paint_cell(start_x as u16, start_y as u16) {
            return;
        }

        let idx = self.index(start_x as u16, start_y as u16);
        self.cells[idx] = ScreenCell {
            ch,
            fg: RenderColor::Crt(fg),
            bg: RenderColor::Crt(bg),
        };
        self.mark_damage_rect(WindowRect {
            left: start_x as u16,
            top: start_y as u16,
            right: start_x as u16,
            bottom: start_y as u16,
        });
    }
}
