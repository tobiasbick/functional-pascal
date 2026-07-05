//! Modal `ListBox` view that mirrors selection into the FPAS host cell.
//!
//! **Documentation:** `docs/pascal/std/tui/app/modals.md`

use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Palette;
use turbo_vision::core::palette_chain::PaletteChainNode;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::listbox::ListBox;
use turbo_vision::views::view::View;

/// Turbo Vision list box wired to a shared FPAS selection cell.
pub(in crate::vm::execute::io::tui) struct BridgedListBox {
    inner: ListBox,
    selection_cell: TurboVisionListSelectionCell,
}

impl BridgedListBox {
    /// Build a modal list box seeded from `selection_cell`.
    pub fn new(
        bounds: Rect,
        items: Vec<String>,
        command_id: u16,
        selection_cell: TurboVisionListSelectionCell,
    ) -> Self {
        let mut inner = ListBox::new(bounds, command_id);
        inner.set_items(items);
        if let Some(selection) = selection_cell.read() {
            inner.set_selection(selection);
        }
        let mut bridged = Self {
            inner,
            selection_cell,
        };
        bridged.sync_selection();
        bridged
    }

    fn sync_selection(&mut self) {
        self.selection_cell.set(self.inner.get_selection());
    }

    /// Replace list items and selection from FPAS host state (live patch path).
    pub(in crate::vm::execute::io::tui) fn set_items_from_fpas(
        &mut self,
        items: Vec<String>,
        selection: Option<usize>,
    ) {
        self.inner.set_items(items);
        if let Some(index) = selection {
            self.inner.set_selection(index);
        }
        self.sync_selection();
    }
}

impl View for BridgedListBox {
    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.inner.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.inner.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.inner.handle_event(event);
        self.sync_selection();
    }

    fn can_focus(&self) -> bool {
        self.inner.can_focus()
    }

    fn state(&self) -> StateFlags {
        self.inner.state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.inner.set_state(state);
    }

    fn set_list_selection(&mut self, index: usize) {
        self.inner.set_list_selection(index);
        self.sync_selection();
    }

    fn get_list_selection(&self) -> usize {
        self.inner.get_list_selection()
    }

    fn set_palette_chain(&mut self, node: Option<PaletteChainNode>) {
        self.inner.set_palette_chain(node);
    }

    fn get_palette_chain(&self) -> Option<&PaletteChainNode> {
        self.inner.get_palette_chain()
    }

    fn get_palette(&self) -> Option<Palette> {
        self.inner.get_palette()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::event::{Event, KB_DOWN};

    #[test]
    fn keyboard_navigation_syncs_selection_cell() {
        let selection_cell = TurboVisionListSelectionCell::new(None);
        let mut list_box = BridgedListBox::new(
            Rect::new(0, 0, 20, 4),
            vec!["alpha".into(), "beta".into(), "gamma".into()],
            100,
            selection_cell.clone(),
        );

        assert_eq!(selection_cell.read(), Some(0));

        let mut event = Event::keyboard(KB_DOWN);
        list_box.handle_event(&mut event);

        assert_eq!(selection_cell.read(), Some(1));
    }
}
