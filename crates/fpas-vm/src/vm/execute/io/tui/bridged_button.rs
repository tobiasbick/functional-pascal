//! `Button` view wrapper with `as_any_mut` for live `SetText` patching.
//!
//! Upstream `Button` has no runtime title setter; the bridge rebuilds the inner view in place.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use turbo_vision::core::command::CommandId;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Palette;
use turbo_vision::core::palette_chain::PaletteChainNode;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::button::Button;
use turbo_vision::views::view::View;

/// Turbo Vision button that can be patched from FPAS `SetText` without a desktop rebuild.
pub(in crate::vm::execute::io::tui) struct BridgedButton {
    inner: Button,
    command: CommandId,
    is_default: bool,
}

impl BridgedButton {
    /// Build a button seeded from `title` and `command`.
    pub fn new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self {
        Self {
            inner: Button::new(bounds, title, command, is_default),
            command,
            is_default,
        }
    }

    /// Push FPAS text into the upstream button (live patch path).
    pub(in crate::vm::execute::io::tui) fn set_text_from_fpas(&mut self, text: &str) {
        let bounds = self.inner.bounds();
        let disabled = self.inner.is_disabled();
        let mut inner = Button::new(bounds, text, self.command, self.is_default);
        if disabled {
            inner.set_disabled(true);
        }
        self.inner = inner;
    }
}

impl View for BridgedButton {
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
    use turbo_vision::core::geometry::Rect;

    #[test]
    fn set_text_from_fpas_updates_without_panic() {
        let bounds = Rect::new(0, 0, 10, 2);
        let mut button = BridgedButton::new(bounds, "OLD", 1, false);
        button.set_text_from_fpas("NEW");
    }
}
