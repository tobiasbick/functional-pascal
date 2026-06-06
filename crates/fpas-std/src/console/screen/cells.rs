use super::{ConsoleState, RenderColor, ScreenCell, WindowRect};

impl ConsoleState {
    /// Fill a zero-based terminal rectangle with one CRT foreground/background pair.
    pub(in super::super) fn fill_rect_crt(
        &mut self,
        rect: crate::ViewRect,
        fg: u8,
        bg: u8,
        ch: char,
    ) {
        if rect.width <= 0 || rect.height <= 0 {
            return;
        }

        let Some(window) = WindowRect::from_zero_based_rect(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            self.width,
            self.height,
        ) else {
            return;
        };

        let cell = ScreenCell {
            ch,
            fg: RenderColor::Crt(fg),
            bg: RenderColor::Crt(bg),
        };

        for y in window.top..=window.bottom {
            for x in window.left..=window.right {
                let idx = self.index(x, y);
                self.cells[idx] = cell;
            }
        }

        self.mark_damage_rect(window);
    }
}
