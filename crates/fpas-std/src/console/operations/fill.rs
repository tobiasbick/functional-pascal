use super::super::Console;
use crate::ViewRect;

impl Console {
    /// Fill a zero-based terminal rectangle during hosted TUI paint.
    pub(crate) fn fill_rect_crt(&mut self, rect: ViewRect, fg: u8, bg: u8, ch: char) {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.fill_rect_crt(rect, fg, bg, ch);
    }
}
