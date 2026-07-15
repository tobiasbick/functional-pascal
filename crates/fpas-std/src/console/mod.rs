//! `Std.Console` - output, line-buffered text input (`Read` / `ReadLn`), and CRT-style
//! keyboard input (`ReadKey` / `KeyPressed`).
//!
//! **Documentation:** `docs/pascal/std/console/README.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file in sync with this module, `crates/fpas-vm/src/vm.rs`
//! (console intrinsics), and `crates/fpas-compiler/src/compiler.rs` (`Write` / `WriteLn` / read intrinsics).

mod cell;
mod input;
mod key_input;
mod operations;
mod render;
mod screen;
mod snapshot;
mod validation;

#[cfg(test)]
mod tests;

pub use cell::{
    CONSOLE_COLOR_KIND_VARIANTS, ConsoleCell, ConsoleColor, ConsoleRect, SavedRegionId,
};
pub use input::{ReadLnQueue, TextInput, read_line_from_stdin};
pub use key_input::KeyInput;
pub use snapshot::ScreenSnapshot;
pub use validation::validate_packed_crt_color;

use screen::{ConsoleState, DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH};
use std::io::Write;

/// Captured output from program execution (for testing).
#[derive(Debug, Clone, Default)]
pub struct CapturedOutput {
    pub lines: Vec<String>,
}

/// Standard console I/O.
///
/// Handles `Std.Console.Write` and `Std.Console.WriteLn`.
/// Output is always captured (for test assertions). When a writer
/// is attached it is also streamed there (for CLI / real execution).
pub struct Console {
    captured: CapturedOutput,
    /// Fragments from `Write` not yet ended by `WriteLn`; one logical line for capture.
    capture_line_buf: String,
    state: ConsoleState,
    writer: Option<Box<dyn Write + Send>>,
    frame_depth: u32,
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl Console {
    pub fn new() -> Self {
        Self {
            captured: CapturedOutput::default(),
            capture_line_buf: String::new(),
            state: ConsoleState::new(DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT),
            writer: None,
            frame_depth: 0,
        }
    }

    pub fn with_writer(writer: Box<dyn Write + Send>) -> Self {
        let (width, height) =
            crossterm::terminal::size().unwrap_or((DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT));
        Self {
            captured: CapturedOutput::default(),
            capture_line_buf: String::new(),
            state: ConsoleState::new(width, height),
            writer: Some(writer),
            frame_depth: 0,
        }
    }

    /// Access captured output (for test assertions).
    pub fn output(&self) -> &CapturedOutput {
        &self.captured
    }

    /// Returns the current logical CRT screen as a text grid snapshot.
    pub fn screen_snapshot(&self) -> ScreenSnapshot {
        ScreenSnapshot::from_state(&self.state)
    }

    /// Returns the character content of one screen row (`y` is one-based).
    ///
    /// **Documentation:** `docs/pascal/std/testing/test.md` (`Std.Test.AssertScreenLine`)
    pub fn query_screen_line(&self, y: u16) -> String {
        self.state.row_text(y)
    }

    /// Returns one CRT cell (`x`/`y` one-based) as `(ch, fg, bg)` with packed colors `0..=15`.
    ///
    /// **Documentation:** `docs/pascal/std/testing/test.md` (`Std.Test.AssertScreenCell`)
    pub fn query_screen_cell(&self, x: u16, y: u16) -> Option<(char, u8, u8)> {
        self.state.packed_cell_at(x, y)
    }

    pub(crate) fn has_terminal_writer(&self) -> bool {
        self.writer.is_some()
    }

    /// Paint one CRT cell for headless Turbo Vision reconciliation (no terminal I/O).
    #[doc(hidden)]
    pub fn paint_headless_cell(&mut self, x: u16, y: u16, ch: char, fg: u8, bg: u8) {
        self.state.paint_packed_cell(x, y, ch, fg, bg);
    }

    /// Clear the logical CRT buffer without emitting terminal output.
    #[doc(hidden)]
    pub fn clear_headless_screen(&mut self) {
        self.state.clear_window();
    }
}

#[cfg(test)]
impl Console {
    pub(crate) fn test_line_text(&self, y: u16) -> String {
        self.state.line_text(y)
    }

    pub(crate) fn test_cell(&self, x: u16, y: u16) -> (char, u8, u8) {
        self.state.cell_at_packed(x, y)
    }

    pub(crate) fn test_cell_colors(&self, x: u16, y: u16) -> (char, String, String) {
        self.state.cell_color_labels(x, y)
    }
}
