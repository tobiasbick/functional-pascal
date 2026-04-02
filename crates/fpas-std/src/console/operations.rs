//! Console operations for `Std.Console`.
//!
//! Spec: `docs/pascal/std/console.md` (from repository root).

use super::Console;
use crate::console::screen::{
    TEXT_MODE_BW40, TEXT_MODE_BW80, TEXT_MODE_C40, TEXT_MODE_C80, TEXT_MODE_CO40, TEXT_MODE_CO80,
    TEXT_MODE_MONO, WindowRect,
};
use crate::error::{StdError, std_runtime_error};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{Command, QueueableCommand};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::io::Write;
use std::thread;
use std::time::Duration;

impl Console {
    pub fn sync_terminal_size(&mut self) {
        if self.writer.is_none() {
            return;
        }
        if let Ok((width, height)) = crossterm::terminal::size() {
            self.state.resize(width, height);
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.state.resize(width, height);
    }

    /// `Std.Console.Write(value)` - print without newline.
    pub fn write(&mut self, value: &Value, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        let s = format!("{value}");
        self.state.write_text(&s, false);
        if self.state.crt_mode {
            self.render_screen(location)?;
        } else if let Some(writer) = &mut self.writer {
            write!(writer, "{s}").map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("Write failed: {e}"),
                    "Check stdout availability and try again.",
                    location,
                )
            })?;
            writer.flush().map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("Write flush failed: {e}"),
                    "Check stdout availability and try again.",
                    location,
                )
            })?;
        }
        self.capture_line_buf.push_str(&s);
        Ok(())
    }

    /// `Std.Console.WriteLn(value)` - print with newline.
    pub fn write_ln(&mut self, value: &Value, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        let s = format!("{value}");
        self.capture_line_buf.push_str(&s);
        let line = std::mem::take(&mut self.capture_line_buf);
        self.captured.lines.push(line);
        self.state.write_text(&s, true);
        if self.state.crt_mode {
            self.render_screen(location)?;
        } else if let Some(writer) = &mut self.writer {
            writeln!(writer, "{s}").map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("WriteLn failed: {e}"),
                    "Check stdout availability and try again.",
                    location,
                )
            })?;
            writer.flush().map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("WriteLn flush failed: {e}"),
                    "Check stdout availability and try again.",
                    location,
                )
            })?;
        }
        Ok(())
    }

    pub fn clr_scr(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.clear_window();
        self.render_screen(location)
    }

    pub fn clr_eol(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.clear_eol();
        self.render_screen(location)
    }

    pub fn goto_xy(&mut self, x: i64, y: i64, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        let x = self.validate_relative_coord(x, self.state.window_width(), "X", location)?;
        let y = self.validate_relative_coord(y, self.state.window_height(), "Y", location)?;
        self.state.set_cursor(x, y);
        self.render_screen(location)
    }

    pub fn where_x(&self) -> i64 {
        i64::from(self.state.cursor_x)
    }

    pub fn where_y(&self) -> i64 {
        i64::from(self.state.cursor_y)
    }

    /// `Std.Console.WindMin` as packed coordinate (low byte: X, high byte: Y).
    /// Spec: `docs/pascal/std/console.md`.
    pub fn wind_min(&self) -> i64 {
        self.state.wind_min()
    }

    /// `Std.Console.WindMax` as packed coordinate (low byte: X, high byte: Y).
    /// Spec: `docs/pascal/std/console.md`.
    pub fn wind_max(&self) -> i64 {
        self.state.wind_max()
    }

    pub fn del_line(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.del_line();
        self.render_screen(location)
    }

    pub fn ins_line(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.ins_line();
        self.render_screen(location)
    }

    pub fn window(
        &mut self,
        x1: i64,
        y1: i64,
        x2: i64,
        y2: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        let x1 = self.validate_absolute_coord(x1, self.state.width, "X1", location)?;
        let y1 = self.validate_absolute_coord(y1, self.state.height, "Y1", location)?;
        let x2 = self.validate_absolute_coord(x2, self.state.width, "X2", location)?;
        let y2 = self.validate_absolute_coord(y2, self.state.height, "Y2", location)?;
        if x1 > x2 || y1 > y2 {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Window({x1}, {y1}, {x2}, {y2}) is invalid"),
                "Use inclusive screen coordinates where X1 <= X2 and Y1 <= Y2.",
                location,
            ));
        }

        self.state.set_window(WindowRect {
            left: x1,
            top: y1,
            right: x2,
            bottom: y2,
        });
        self.render_screen(location)
    }

    /// `Std.Console.TextColor(Color)` — select a packed CRT foreground color.
    pub fn text_color(&mut self, color: i64, location: SourceLocation) -> Result<(), StdError> {
        self.enable_crt_mode();
        self.state.fg = self.validate_color(color, "TextColor", location)?;
        self.state.use_packed_colors();
        Ok(())
    }

    /// `Std.Console.TextBackground(Color)` — select a packed CRT background color.
    pub fn text_background(
        &mut self,
        color: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.enable_crt_mode();
        self.state.bg = self.validate_color(color, "TextBackground", location)?;
        self.state.use_packed_colors();
        Ok(())
    }

    /// `Std.Console.HighVideo()` — enable the packed bright foreground bit.
    pub fn high_video(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.fg |= 0x08;
        self.state.use_packed_colors();
        self.render_screen(location)
    }

    /// `Std.Console.LowVideo()` — disable the packed bright foreground bit.
    pub fn low_video(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.fg &= 0x07;
        self.state.use_packed_colors();
        self.render_screen(location)
    }

    /// `Std.Console.NormVideo()` — restore packed light-gray-on-black colors.
    pub fn norm_video(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.fg = 7;
        self.state.bg = 0;
        self.state.use_packed_colors();
        self.render_screen(location)
    }

    pub fn text_attr(&self) -> i64 {
        i64::from((self.state.bg << 4) | (self.state.fg & 0x0F))
    }

    /// `Std.Console.SetTextAttr(Attr)` — restore packed CRT colors from `Attr`.
    pub fn set_text_attr(&mut self, attr: i64, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        let attr = self.validate_text_attr(attr, location)?;
        self.state.fg = attr & 0x0F;
        self.state.bg = (attr >> 4) & 0x0F;
        self.state.use_packed_colors();
        self.render_screen(location)
    }

    pub fn delay(&mut self, ms: i64, location: SourceLocation) -> Result<(), StdError> {
        if ms < 0 {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Delay expects a non-negative millisecond count, got {ms}"),
                "Pass `0` or a positive integer number of milliseconds.",
                location,
            ));
        }
        thread::sleep(Duration::from_millis(ms as u64));
        Ok(())
    }

    pub fn cursor_on(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.cursor_visible = true;
        self.state.cursor_big = false;
        self.render_screen(location)
    }

    pub fn cursor_off(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.cursor_visible = false;
        self.render_screen(location)
    }

    pub fn cursor_big(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.cursor_visible = true;
        self.state.cursor_big = true;
        self.render_screen(location)
    }

    /// `Std.Console.TextMode(Mode)` — reset packed CRT state and clear the screen.
    pub fn text_mode(&mut self, mode: i64, location: SourceLocation) -> Result<(), StdError> {
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.validate_text_mode(mode, location)?;
        self.state.last_mode = mode;
        self.state
            .set_window(WindowRect::full(self.state.width, self.state.height));
        self.state.fg = 7;
        self.state.bg = 0;
        self.state.use_packed_colors();
        self.state.cursor_visible = true;
        self.state.cursor_big = false;
        self.state.clear_window();
        self.render_screen(location)
    }

    pub fn last_mode(&self) -> i64 {
        self.state.last_mode
    }

    pub fn screen_width(&self) -> i64 {
        if self.writer.is_some()
            && let Ok((width, _)) = crossterm::terminal::size()
        {
            return i64::from(width);
        }
        self.state.screen_width()
    }

    pub fn screen_height(&self) -> i64 {
        if self.writer.is_some()
            && let Ok((_, height)) = crossterm::terminal::size()
        {
            return i64::from(height);
        }
        self.state.screen_height()
    }

    pub fn sound(&mut self, hz: i64, location: SourceLocation) -> Result<(), StdError> {
        if hz <= 0 {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Sound expects a positive frequency in Hz, got {hz}"),
                "Pass a value greater than 0, for example `Sound(440)`.",
                location,
            ));
        }
        if let Some(writer) = &mut self.writer {
            write!(writer, "\u{0007}").map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("Sound failed: {e}"),
                    "Check stdout availability and try again.",
                    location,
                )
            })?;
            writer.flush().map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("Sound flush failed: {e}"),
                    "Check stdout availability and try again.",
                    location,
                )
            })?;
        }
        Ok(())
    }

    pub fn no_sound(&mut self) -> Result<(), StdError> {
        Ok(())
    }

    pub fn assign_crt(&mut self) -> Result<(), StdError> {
        self.enable_crt_mode();
        Ok(())
    }

    pub fn enter_alt_screen(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.enable_crt_mode();
        self.run_writer_command(EnterAlternateScreen, "EnterAltScreen failed", location)
    }

    pub fn leave_alt_screen(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(LeaveAlternateScreen, "LeaveAltScreen failed", location)
    }

    pub fn enable_mouse(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(EnableMouseCapture, "EnableMouse failed", location)
    }

    pub fn disable_mouse(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(DisableMouseCapture, "DisableMouse failed", location)
    }

    pub fn enable_focus(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(EnableFocusChange, "EnableFocus failed", location)
    }

    pub fn disable_focus(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(DisableFocusChange, "DisableFocus failed", location)
    }

    pub fn enable_paste(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(EnableBracketedPaste, "EnablePaste failed", location)
    }

    pub fn disable_paste(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.run_writer_command(DisableBracketedPaste, "DisablePaste failed", location)
    }

    /// `Std.Console.TextColorRGB(R, G, B)` — set fg to 24-bit truecolor.
    ///
    /// Spec: `docs/pascal/std/console.md`.
    pub fn text_color_rgb(
        &mut self,
        r: i64,
        g: i64,
        b: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let (r, g, b) = self.validate_rgb(r, g, b, "TextColorRGB", location)?;
        self.state.set_extended_fg_rgb(r, g, b);
        self.run_writer_command(
            crossterm::style::SetForegroundColor(crossterm::style::Color::Rgb { r, g, b }),
            "TextColorRGB failed",
            location,
        )
    }

    /// `Std.Console.TextBackgroundRGB(R, G, B)` — set bg to 24-bit truecolor.
    ///
    /// Spec: `docs/pascal/std/console.md`.
    pub fn text_background_rgb(
        &mut self,
        r: i64,
        g: i64,
        b: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let (r, g, b) = self.validate_rgb(r, g, b, "TextBackgroundRGB", location)?;
        self.state.set_extended_bg_rgb(r, g, b);
        self.run_writer_command(
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Rgb { r, g, b }),
            "TextBackgroundRGB failed",
            location,
        )
    }

    /// `Std.Console.TextColor256(Index)` — set fg to 256-color palette index (0–255).
    ///
    /// Spec: `docs/pascal/std/console.md`.
    pub fn text_color_256(&mut self, index: i64, location: SourceLocation) -> Result<(), StdError> {
        let index = self.validate_color_256(index, "TextColor256", location)?;
        self.state.set_extended_fg_ansi(index);
        self.run_writer_command(
            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(index)),
            "TextColor256 failed",
            location,
        )
    }

    /// `Std.Console.TextBackground256(Index)` — set bg to 256-color palette index (0–255).
    ///
    /// Spec: `docs/pascal/std/console.md`.
    pub fn text_background_256(
        &mut self,
        index: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let index = self.validate_color_256(index, "TextBackground256", location)?;
        self.state.set_extended_bg_ansi(index);
        self.run_writer_command(
            crossterm::style::SetBackgroundColor(crossterm::style::Color::AnsiValue(index)),
            "TextBackground256 failed",
            location,
        )
    }

    fn enable_crt_mode(&mut self) {
        self.state.crt_mode = true;
    }

    fn validate_text_mode(&self, mode: i64, location: SourceLocation) -> Result<(), StdError> {
        if mode < 0 {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("TextMode expects a non-negative mode value, got {mode}"),
                "Use a CRT mode constant such as `C80`, `BW80`, or another non-negative integer.",
                location,
            ));
        }

        if matches!(
            mode,
            TEXT_MODE_BW40
                | TEXT_MODE_C40
                | TEXT_MODE_BW80
                | TEXT_MODE_C80
                | TEXT_MODE_CO40
                | TEXT_MODE_CO80
                | TEXT_MODE_MONO
                | 256
        ) {
            return Ok(());
        }

        Ok(())
    }

    fn run_writer_command<C: Command>(
        &mut self,
        command: C,
        context: &str,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let Some(writer) = &mut self.writer else {
            return Ok(());
        };
        writer.queue(command).map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{context}: {e}"),
                "Run this in a terminal that supports screen control sequences.",
                location,
            )
        })?;
        writer.flush().map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{context} (flush): {e}"),
                "Check stdout availability and try again.",
                location,
            )
        })
    }
}
