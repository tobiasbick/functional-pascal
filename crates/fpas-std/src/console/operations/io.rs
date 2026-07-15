use super::super::Console;
use crate::error::{StdError, std_runtime_error};
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
            self.render_if_ready(location)?;
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
            self.render_if_ready(location)?;
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
}
