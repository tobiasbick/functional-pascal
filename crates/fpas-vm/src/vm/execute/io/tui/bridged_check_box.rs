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
        if super::check_box_mouse::try_toggle_check_box_on_mouse_down(&mut self.inner, event) {
            self.sync_checked();
            return;
        }
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
}
