//! Native TUI query intrinsics (Phase 3–4).
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Worker {
    /// Executes read-only native TUI query intrinsics.
    pub(super) fn try_exec_tui_query_host_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::QueryScreenSize) => {
                self.pop_tui_application(line)?;
                let (width, height) =
                    self.with_console(|console| (console.screen_width(), console.screen_height()));
                self.push(Self::tui_size_record(width, height))?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryScreenLine) => {
                let y = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let y = Self::screen_row_to_u16(y, line)?;
                let row = self.with_console(|console| -> Result<String, VmError> {
                    Self::validate_screen_row(console.screen_height(), y, line)?;
                    Ok(console.query_screen_line(y))
                })?;
                self.push(Value::Str(row))?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryScreenCell) => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let x = Self::screen_column_to_u16(x, line)?;
                let y = Self::screen_row_to_u16(y, line)?;
                let (ch, fg, bg) =
                    self.with_console(|console| -> Result<(char, u8, u8), VmError> {
                        Self::validate_screen_cell(
                            console.screen_width(),
                            console.screen_height(),
                            x,
                            y,
                            line,
                        )?;
                        console
                            .query_screen_cell(x, y)
                            .ok_or_else(|| query_cell_error(x, y, line))
                    })?;
                self.push(Self::tui_screen_cell_record(ch, fg, bg))?;
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
                    "Application.QueryScreenLine(App, Y) requires Y in 1..={}, got {y}.",
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
                    "Application.QueryScreenCell(App, X, Y) requires X in 1..={}, got {x}.",
                    u16::MAX
                ),
                "Pass one-based column coordinates within the virtual screen width.",
                line,
            ));
        }
        Ok(x as u16)
    }

    fn validate_screen_row(
        screen_height: i64,
        y: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if i64::from(y) > screen_height {
            return Err(screen_row_out_of_range(y, screen_height, line));
        }
        Ok(())
    }

    fn validate_screen_cell(
        screen_width: i64,
        screen_height: i64,
        x: u16,
        y: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        Self::validate_screen_row(screen_height, y, line)?;
        if i64::from(x) > screen_width {
            return Err(screen_coord_out_of_range("X", x, screen_width, line));
        }
        Ok(())
    }
}

fn screen_row_out_of_range(y: u16, limit: i64, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "Application.QueryScreenLine(App, Y) coordinate Y={y} is outside the virtual screen (1..={limit})."
        ),
        "Query rows inside the painted screen bounds; use Application.QueryScreenSize for height.",
        line,
    )
}

fn screen_coord_out_of_range(axis: &str, value: u16, limit: i64, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "Application.QueryScreenCell(App, X, Y) coordinate {axis}={value} is outside the virtual screen (1..={limit})."
        ),
        "Query cells inside the painted screen bounds; use Application.QueryScreenSize for width and height.",
        line,
    )
}

fn query_cell_error(x: u16, y: u16, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "Application.QueryScreenCell(App, {x}, {y}) is out of range or uses non-CRT colors."
        ),
        "Query cells inside the virtual screen after paint; v1 supports packed CRT colors only (0..=15).",
        line,
    )
}
