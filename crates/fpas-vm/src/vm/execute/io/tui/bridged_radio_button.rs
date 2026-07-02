//! Modal `RadioButton` view that mirrors selected state into the FPAS host cell.
//!
//! **Documentation:** `docs/pascal/std/tui/app/modals.md`

use crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Palette;
use turbo_vision::core::palette_chain::PaletteChainNode;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::radiobutton::RadioButton;
use turbo_vision::views::view::View;

/// Turbo Vision radio button wired to shared FPAS group selection cells.
pub(in crate::vm::execute::io::tui) struct BridgedRadioButton {
    inner: RadioButton,
    selected_cell: TurboVisionBoolCell,
    group_cells: Vec<TurboVisionBoolCell>,
    tree_dirty: TurboVisionBoolCell,
}

impl BridgedRadioButton {
    /// Build a modal radio button seeded from `selected_cell`.
    pub fn new(
        bounds: Rect,
        label: &str,
        group_id: u16,
        selected_cell: TurboVisionBoolCell,
        group_cells: Vec<TurboVisionBoolCell>,
        tree_dirty: TurboVisionBoolCell,
    ) -> Self {
        let mut inner = RadioButton::new(bounds, label, group_id);
        inner.set_selected(selected_cell.read());
        Self {
            inner,
            selected_cell,
            group_cells,
            tree_dirty,
        }
    }

    fn sync_selected(&mut self) {
        if self.inner.is_selected() {
            for cell in &self.group_cells {
                cell.set(false);
            }
            self.selected_cell.set(true);
        } else {
            self.selected_cell.set(false);
        }
    }
}

impl View for BridgedRadioButton {
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
        if super::radio_button_mouse::try_select_radio_button_on_mouse_down(&mut self.inner, event)
        {
            self.sync_selected();
            self.tree_dirty.set(true);
            return;
        }
        self.inner.handle_event(event);
        self.sync_selected();
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

    fn set_palette_chain(&mut self, node: Option<PaletteChainNode>) {
        self.inner.set_palette_chain(node);
    }

    fn get_palette_chain(&self) -> Option<&PaletteChainNode> {
        self.inner.get_palette_chain()
    }

    fn get_palette(&self) -> Option<Palette> {
        self.inner.get_palette()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::event::{Event, EventType, MB_LEFT_BUTTON};
    use turbo_vision::core::geometry::Rect;

    #[test]
    fn mouse_down_selects_radio_and_syncs_group_cells() {
        let first_cell = TurboVisionBoolCell::new(true);
        let second_cell = TurboVisionBoolCell::new(false);
        let tree_dirty = TurboVisionBoolCell::new(false);
        let bounds = Rect::new(0, 0, 12, 1);
        let mut second = BridgedRadioButton::new(
            bounds,
            "second",
            3,
            second_cell.clone(),
            vec![first_cell.clone(), second_cell.clone()],
            tree_dirty.clone(),
        );

        let mut event = Event::mouse(EventType::MouseDown, bounds.a, MB_LEFT_BUTTON, false);
        second.handle_event(&mut event);

        assert!(!first_cell.read());
        assert!(second_cell.read());
        assert!(tree_dirty.read());
    }
}
