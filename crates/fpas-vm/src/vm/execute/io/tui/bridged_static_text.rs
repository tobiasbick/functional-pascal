//! `StaticText` view wrapper with `as_any_mut` for live `SetText` patching.
//!
//! Upstream `StaticText` has no runtime text setter; the bridge rebuilds the inner view in place.
//!
//! **Documentation:** `docs/refactor/tui-bridge/05-reduce-reconcile-rebuild.md`

use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Palette;
use turbo_vision::core::palette_chain::PaletteChainNode;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::view::View;

/// Turbo Vision static text that can be patched from FPAS `SetText` without a desktop rebuild.
pub(in crate::vm::execute::io::tui) struct BridgedStaticText {
    inner: StaticText,
}

impl BridgedStaticText {
    /// Build left-aligned static text seeded from `text`.
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            inner: StaticText::new(bounds, text),
        }
    }

    /// Push FPAS text into the upstream static text (live patch path).
    pub(in crate::vm::execute::io::tui) fn set_text_from_fpas(&mut self, text: &str) {
        let bounds = self.inner.bounds();
        self.inner = StaticText::new(bounds, text);
    }
}

impl View for BridgedStaticText {
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

    fn set_palette_chain(&mut self, node: Option<PaletteChainNode>) {
        self.inner.set_palette_chain(node);
    }

    fn get_palette_chain(&self) -> Option<&PaletteChainNode> {
        self.inner.get_palette_chain()
    }

    fn get_palette(&self) -> Option<Palette> {
        self.inner.get_palette()
    }

    fn state(&self) -> StateFlags {
        self.inner.state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.inner.set_state(state);
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
        let bounds = Rect::new(0, 0, 20, 1);
        let mut static_text = BridgedStaticText::new(bounds, "OLD");
        static_text.set_text_from_fpas("NEW");
    }
}
