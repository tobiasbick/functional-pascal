//! Console text output helpers for hosted TUI widgets.
//!
//! Spec: `docs/pascal/std/console.md` (from repository root).

use super::super::Console;

impl Console {
    /// Write `text` at a zero-based terminal cell using CRT color indices.
    pub(crate) fn write_text_at_crt(&mut self, x: i64, y: i64, text: &str, fg: u8, bg: u8) {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.write_text_at_crt(x, y, text, fg, bg);
    }
}
