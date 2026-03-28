use super::ConsoleState;

impl ConsoleState {
    /// Returns `true` when no previous frame has been committed yet.
    pub(in super::super) fn is_first_frame(&self) -> bool {
        self.prev_cells.is_empty()
    }

    /// Returns `true` when the cell at `(x, y)` differs from the previous frame.
    pub(in super::super) fn cell_changed(&self, x: u16, y: u16) -> bool {
        let idx = self.index(x, y);
        self.prev_cells.get(idx) != Some(&self.cells[idx])
    }

    /// Snapshot the current cells as the previous frame.
    pub(in super::super) fn commit_frame(&mut self) {
        self.prev_cells.clone_from(&self.cells);
    }
}
