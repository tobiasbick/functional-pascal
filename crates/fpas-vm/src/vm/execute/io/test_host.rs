//! VM execution for `Std.Test` intrinsics that read console or TUI state.
//!
//! **Documentation:** `docs/pascal/std/testing/test.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{Intrinsic, SourceLocation, TestIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::{assert_screen_cell, assert_screen_line};

impl Worker {
    /// Executes `Std.Test` intrinsics that need hosted console or view state.
    pub(super) fn try_exec_test_host_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Test(TestIntrinsic::AssertScreenLine) => {
                let y = self.pop_int(line)?;
                let expected = self.pop_string(line)?;
                let y = Self::screen_row_to_u16(y, line)?;
                let actual = self.with_console(|console| console.query_screen_line(y));
                assert_screen_line(expected, actual, line)?;
            }
            Intrinsic::Test(TestIntrinsic::AssertScreenCell) => {
                let expected_bg = self.pop_int(line)?;
                let expected_fg = self.pop_int(line)?;
                let expected_ch = self.pop_char(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                let x = Self::screen_column_to_u16(x, line)?;
                let y = Self::screen_row_to_u16(y, line)?;
                let (actual_ch, actual_fg, actual_bg) = self.with_console(|console| {
                    console
                        .query_screen_cell(x, y)
                        .ok_or_else(|| query_cell_error(x, y, line))
                })?;
                assert_screen_cell(
                    expected_ch,
                    expected_fg,
                    expected_bg,
                    actual_ch,
                    actual_fg,
                    actual_bg,
                    line,
                )?;
            }
            Intrinsic::Test(TestIntrinsic::PushReadLn) => {
                let line_text = self.pop_string(line)?;
                self.with_text_input(|input| input.push_line(&line_text));
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    pub(in crate::vm::execute::io) fn screen_row_to_u16(
        y: i64,
        line: SourceLocation,
    ) -> Result<u16, VmError> {
        if y <= 0 || y > i64::from(u16::MAX) {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!(
                    "Screen row coordinate must be in 1..={}, got {y}.",
                    u16::MAX
                ),
                "Pass a one-based row index within the virtual screen height.",
                line,
            ));
        }
        Ok(y as u16)
    }

    pub(in crate::vm::execute::io) fn screen_column_to_u16(
        x: i64,
        line: SourceLocation,
    ) -> Result<u16, VmError> {
        if x <= 0 || x > i64::from(u16::MAX) {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!(
                    "Screen column coordinate must be in 1..={}, got {x}.",
                    u16::MAX
                ),
                "Pass one-based column coordinates within the virtual screen width.",
                line,
            ));
        }
        Ok(x as u16)
    }

    fn pop_string(&mut self, line: SourceLocation) -> Result<String, VmError> {
        match self.pop(line)? {
            Value::Str(text) => Ok(text),
            other => Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!(
                    "Std.Test screen assertion expected string, got {}",
                    other.type_name()
                ),
                "Pass a string literal or variable for the expected screen line.",
                line,
            )),
        }
    }

    fn pop_char(&mut self, line: SourceLocation) -> Result<char, VmError> {
        match self.pop(line)? {
            Value::Str(text) => {
                let mut chars = text.chars();
                match (chars.next(), chars.next()) {
                    (Some(ch), None) => Ok(ch),
                    (None, _) => Err(runtime_error(
                        RUNTIME_CONSOLE_STATE_ERROR,
                        "Std.Test.AssertScreenCell expected a single-character string",
                        "Pass a single-character string for the expected cell character.",
                        line,
                    )),
                    _ => Err(runtime_error(
                        RUNTIME_CONSOLE_STATE_ERROR,
                        format!(
                            "Std.Test.AssertScreenCell expected a single-character string, got `{text}`"
                        ),
                        "Pass a single-character string for the expected cell character.",
                        line,
                    )),
                }
            }
            other => Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!(
                    "Std.Test.AssertScreenCell expected string, got {}",
                    other.type_name()
                ),
                "Pass a single-character string for the expected cell character.",
                line,
            )),
        }
    }
}

fn query_cell_error(x: u16, y: u16, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!("AssertScreenCell({x}, {y}, …) is out of range or uses non-CRT colors."),
        "Assert cells inside the virtual screen after paint; v1 supports packed CRT colors only (0..=15).",
        line,
    )
}
