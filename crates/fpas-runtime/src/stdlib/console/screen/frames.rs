use super::{ConsoleState, FrameDamage, WindowRect};

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

    pub(in super::super) fn full_screen_rect(&self) -> WindowRect {
        WindowRect::full(self.width, self.height)
    }

    pub(in super::super) fn mark_damage_rect(&mut self, rect: WindowRect) {
        self.pending_frame_damage = Some(match self.pending_frame_damage {
            Some(FrameDamage::FullFrame) => FrameDamage::FullFrame,
            Some(FrameDamage::Rect(existing)) => FrameDamage::Rect(existing.union(rect)),
            None => FrameDamage::Rect(rect),
        });
    }

    pub(in super::super) fn take_frame_damage(&mut self) -> Option<FrameDamage> {
        self.pending_frame_damage.take()
    }

    /// Snapshot the current cells as the previous frame.
    pub(in super::super) fn commit_frame(&mut self) {
        self.prev_cells.clone_from(&self.cells);
        self.pending_frame_damage = None;
    }
}
