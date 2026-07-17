use super::{ConsoleState, SavedRegion};
use crate::console::cell::{ConsoleRect, SavedRegionId};
use crate::text::cell_width::grapheme_cell_width;

impl ConsoleState {
    pub(in super::super) fn save_region(&mut self, rect: ConsoleRect) -> Option<SavedRegionId> {
        let mut rect = self.public_rect(rect)?;
        if rect.left > 1
            && (rect.top..=rect.bottom).any(|y| self.cell_at(rect.left, y).continuation)
        {
            rect.left -= 1;
        }
        if rect.right < self.width
            && (rect.top..=rect.bottom)
                .any(|y| grapheme_cell_width(&self.cell_at(rect.right, y).glyph) == Some(2))
        {
            rect.right += 1;
        }

        let mut cells = Vec::with_capacity(usize::from(rect.width()) * usize::from(rect.height()));
        for y in rect.top..=rect.bottom {
            for x in rect.left..=rect.right {
                cells.push(self.cell_at(x, y));
            }
        }

        let id = SavedRegionId(self.next_saved_region_id);
        self.next_saved_region_id = self.next_saved_region_id.saturating_add(1);
        self.saved_regions.insert(id, SavedRegion { rect, cells });
        Some(id)
    }

    pub(in super::super) fn restore_region(&mut self, id: SavedRegionId) -> bool {
        let Some(saved) = self.saved_regions.remove(&id) else {
            return false;
        };
        if saved.rect.right > self.width || saved.rect.bottom > self.height {
            return false;
        }

        let mut cells = saved.cells.into_iter();
        for y in saved.rect.top..=saved.rect.bottom {
            for x in saved.rect.left..=saved.rect.right {
                if let Some(cell) = cells.next() {
                    let index = self.index(x, y);
                    self.cells[index] = cell;
                }
            }
        }
        self.mark_damage_rect(saved.rect);
        true
    }

    pub(in super::super) fn discard_region(&mut self, id: SavedRegionId) -> bool {
        self.saved_regions.remove(&id).is_some()
    }
}
