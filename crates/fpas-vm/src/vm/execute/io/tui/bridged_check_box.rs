//! Modal `CheckBox` view that mirrors checked state into the FPAS host cell.
//!
//! **Documentation:** `docs/pascal/std/tui/app/modals.md`

use crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Palette;
use turbo_vision::core::palette_chain::PaletteChainNode;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::checkbox::CheckBox;
use turbo_vision::views::view::View;

/// Turbo Vision checkbox wired to a shared FPAS checked cell.
pub(in crate::vm::execute::io::tui) struct BridgedCheckBox {
    inner: CheckBox,
    checked_cell: TurboVisionBoolCell,
}

impl BridgedCheckBox {
    /// Build a modal checkbox seeded from `checked_cell`.
    pub fn new(bounds: Rect, label: &str, checked_cell: TurboVisionBoolCell) -> Self {
        let checked = checked_cell.read();
        let mut inner = CheckBox::new(bounds, label);
        inner.set_checked(checked);
        Self {
            inner,
            checked_cell,
        }
    }

    fn sync_checked(&mut self) {
        self.checked_cell.set(self.inner.is_checked());
    }

    /// Copy upstream checkbox state into the host cell (try-2 read-back path).
    pub(in crate::vm::execute::io::tui) fn sync_checked_from_view(&mut self) {
        self.sync_checked();
    }

    /// Push FPAS cell state into the upstream checkbox (live patch path).
    pub(in crate::vm::execute::io::tui) fn sync_from_cell(&mut self) {
        self.inner.set_checked(self.checked_cell.read());
    }

    /// Push FPAS label text into the upstream checkbox (live patch path).
    pub(in crate::vm::execute::io::tui) fn set_text_from_fpas(&mut self, text: &str) {
        let bounds = self.inner.bounds();
        let checked = self.checked_cell.read();
        let mut inner = CheckBox::new(bounds, text);
        inner.set_checked(checked);
        self.inner = inner;
    }
}

impl View for BridgedCheckBox {
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
        self.sync_checked();
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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::event::{Event, EventType, MB_LEFT_BUTTON};
    use turbo_vision::core::geometry::Rect;

    #[test]
    fn mouse_down_toggles_check_box_and_syncs_cell() {
        let checked_cell = TurboVisionBoolCell::new(false);
        let bounds = Rect::new(0, 0, 12, 1);
        let mut check_box = BridgedCheckBox::new(bounds, "opt", checked_cell.clone());

        let mut event = Event::mouse(EventType::MouseDown, bounds.a, MB_LEFT_BUTTON, false);
        check_box.handle_event(&mut event);

        assert!(checked_cell.read());
    }
}
