use super::Console;
use crate::error::{StdError, std_runtime_error};
use crossterm::QueueableCommand;
use crossterm::cursor::{Hide, MoveTo, SetCursorStyle, Show};
use crossterm::style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::io::Write;

impl Console {
    /// Render only the cells that changed since the last frame (differential rendering).
    /// On the very first frame, clear the terminal and draw everything.
    pub(super) fn render_screen(&mut self, location: SourceLocation) -> Result<(), StdError> {
        let Some(writer) = self.writer.as_mut() else {
            return Ok(());
        };

        let first_frame = self.state.is_first_frame();

        let state = &self.state;

        let map_err = |e: std::io::Error| {
            std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Console redraw failed: {e}"),
                "Run this in a terminal that supports screen control sequences.",
                location,
            )
        };

        if first_frame {
            // First frame: clear terminal once and draw every cell.
            writer.queue(Clear(ClearType::All)).map_err(map_err)?;
            for y in 1..=state.height {
                writer.queue(MoveTo(0, y - 1)).map_err(map_err)?;
                for x in 1..=state.width {
                    let cell = state.cell_at(x, y);
                    writer
                        .queue(SetForegroundColor(cell.fg.to_crossterm()))
                        .and_then(|w| w.queue(SetBackgroundColor(cell.bg.to_crossterm())))
                        .and_then(|w| w.queue(Print(cell.ch)))
                        .map_err(map_err)?;
                }
            }
        } else {
            // Subsequent frames: only repaint cells that differ from the previous frame.
            let mut last_fg = None;
            let mut last_bg = None;

            for y in 1..=state.height {
                for x in 1..=state.width {
                    if !state.cell_changed(x, y) {
                        continue;
                    }
                    let cell = state.cell_at(x, y);
                    writer.queue(MoveTo(x - 1, y - 1)).map_err(map_err)?;
                    if last_fg != Some(cell.fg) {
                        writer
                            .queue(SetForegroundColor(cell.fg.to_crossterm()))
                            .map_err(map_err)?;
                        last_fg = Some(cell.fg);
                    }
                    if last_bg != Some(cell.bg) {
                        writer
                            .queue(SetBackgroundColor(cell.bg.to_crossterm()))
                            .map_err(map_err)?;
                        last_bg = Some(cell.bg);
                    }
                    writer.queue(Print(cell.ch)).map_err(map_err)?;
                }
            }
        }

        writer.queue(ResetColor).map_err(map_err)?;

        if state.cursor_visible {
            writer
                .queue(Show)
                .and_then(|w| {
                    if state.cursor_big {
                        w.queue(SetCursorStyle::SteadyBlock)
                    } else {
                        w.queue(SetCursorStyle::DefaultUserShape)
                    }
                })
                .and_then(|w| w.queue(MoveTo(state.abs_x() - 1, state.abs_y() - 1)))
                .map_err(map_err)?;
        } else {
            writer.queue(Hide).map_err(map_err)?;
        }

        writer.flush().map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Console flush failed: {e}"),
                "Check stdout availability and try again.",
                location,
            )
        })?;

        // Snapshot current frame for next diff comparison.
        self.state.commit_frame();
        Ok(())
    }
}
