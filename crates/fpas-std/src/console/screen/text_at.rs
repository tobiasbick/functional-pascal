use super::{ConsoleState, RenderColor, ScreenCell, WindowRect};
use crate::text::{WIDE_CONTINUATION, display_width};

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
            let width = i64::from(display_width(ch));
            if width == 0 {
                continue;
            }
            if col > i64::from(self.width) || start_y > i64::from(self.height) {
                break;
            }
            damage = Self::paint_display_char(self, col, start_y, ch, fg, bg, damage);
            col += width;
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

        if let Some(window) = Self::paint_display_char(self, start_x, start_y, ch, fg, bg, None) {
            self.mark_damage_rect(window);
        }
    }

    fn paint_display_char(
        state: &mut ConsoleState,
        col: i64,
        row: i64,
        ch: char,
        fg: u8,
        bg: u8,
        mut damage: Option<WindowRect>,
    ) -> Option<WindowRect> {
        let width = i64::from(display_width(ch));
        if width == 0 {
            return damage;
        }

        for offset in 0..width {
            let paint_col = col + offset;
            if paint_col < 1
                || row < 1
                || paint_col > i64::from(state.width)
                || row > i64::from(state.height)
            {
                continue;
            }
            if !state.can_paint_cell(paint_col as u16, row as u16) {
                continue;
            }
            let paint_ch = if offset == 0 { ch } else { WIDE_CONTINUATION };
            let idx = state.index(paint_col as u16, row as u16);
            state.cells[idx] = ScreenCell {
                ch: paint_ch,
                fg: RenderColor::Crt(fg),
                bg: RenderColor::Crt(bg),
            };
            let cell = WindowRect {
                left: paint_col as u16,
                top: row as u16,
                right: paint_col as u16,
                bottom: row as u16,
            };
            damage = Some(match damage {
                Some(existing) => existing.union(cell),
                None => cell,
            });
        }

        damage
    }
}
