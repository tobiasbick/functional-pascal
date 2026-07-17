use super::super::{Console, ConsoleCell, ConsoleRect};
use crate::error::{StdError, std_runtime_error};
use crate::text::cell_width::{grapheme_cell_width, str_display_width};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Console {
    /// Paints one logical cell at absolute 1-based screen coordinates.
    ///
    /// **Documentation:** `docs/pascal/std/console/cells-frames.md`.
    pub fn put_cell(
        &mut self,
        x: i64,
        y: i64,
        cell: ConsoleCell,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        Self::validate_cell(&cell, "PutCell", location)?;
        self.sync_terminal_size();
        self.enable_crt_mode();
        let (Some(x), Some(y)) = (
            self.check_coord(x, self.state.width()),
            self.check_coord(y, self.state.height()),
        ) else {
            return Ok(());
        };
        self.state.put_cell(x, y, cell);
        self.render_if_ready(location)
    }

    /// Returns one logical cell at absolute 1-based screen coordinates.
    ///
    /// Continuation columns of wide glyphs and out-of-bounds coordinates return `None`.
    pub fn get_cell(&self, x: i64, y: i64) -> Option<ConsoleCell> {
        let x = self.check_coord(x, self.state.width())?;
        let y = self.check_coord(y, self.state.height())?;
        self.state.public_cell_at(x, y)
    }

    /// Fills a clipped rectangle with a single-column cell.
    pub fn fill_rect(
        &mut self,
        rect: ConsoleRect,
        cell: ConsoleCell,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        Self::validate_cell(&cell, "FillRect", location)?;
        if grapheme_cell_width(&cell.glyph) != Some(1) {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "FillRect requires a glyph that occupies exactly one terminal column",
                "Use a space, box-drawing character, or another single-column glyph.",
                location,
            ));
        }
        self.sync_terminal_size();
        self.enable_crt_mode();
        self.state.fill_rect(rect, cell);
        self.render_if_ready(location)
    }

    /// Paints cells from left to right, advancing by each glyph's display width.
    pub fn write_cells(
        &mut self,
        x: i64,
        y: i64,
        cells: &[ConsoleCell],
        location: SourceLocation,
    ) -> Result<(), StdError> {
        for cell in cells {
            Self::validate_cell(cell, "WriteCells", location)?;
        }
        self.sync_terminal_size();
        self.enable_crt_mode();
        let (Some(x), Some(y)) = (
            self.check_coord(x, self.state.width()),
            self.check_coord(y, self.state.height()),
        ) else {
            return Ok(());
        };
        self.state.write_cells(x, y, cells);
        self.render_if_ready(location)
    }

    /// Returns the terminal-column width of a string.
    pub fn display_width(text: &str) -> i64 {
        str_display_width(text)
    }

    /// Validates one renderable extended grapheme cluster and returns its terminal width.
    ///
    /// **Documentation:** `docs/pascal/std/console/cells-frames.md`.
    pub fn grapheme_width(text: &str, location: SourceLocation) -> Result<i64, StdError> {
        grapheme_cell_width(text).map(i64::from).ok_or_else(|| {
            std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "GraphemeWidth requires one non-zero-width extended grapheme cluster",
                "Pass one printable glyph, optionally with combining marks or joiners.",
                location,
            )
        })
    }

    fn validate_cell(
        cell: &ConsoleCell,
        operation: &str,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if grapheme_cell_width(&cell.glyph).is_none() {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{operation} requires one non-zero-width extended grapheme cluster"),
                "Use one printable glyph, optionally with combining marks or joiners.",
                location,
            ));
        }
        Ok(())
    }
}
