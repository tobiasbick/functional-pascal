//! `TextViewer` view wrapper with `as_any_mut` for live `SetText` patching.
//!
//! **Documentation:** `docs/refactor/tui-bridge/05-reduce-reconcile-rebuild.md`

use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Palette;
use turbo_vision::core::palette_chain::PaletteChainNode;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::text_viewer::TextViewer;
use turbo_vision::views::view::View;

/// Turbo Vision text viewer that can be patched from FPAS `SetText` without a desktop rebuild.
pub(in crate::vm::execute::io::tui) struct BridgedTextViewer {
    inner: TextViewer,
}

impl BridgedTextViewer {
    /// Build a text viewer seeded from `text`.
    pub fn new(bounds: Rect, text: &str) -> Self {
        let mut inner = TextViewer::new(bounds);
        inner.set_text(text);
        Self { inner }
    }

    /// Push FPAS text into the upstream text viewer (live patch path).
    pub(in crate::vm::execute::io::tui) fn set_text_from_fpas(&mut self, text: &str) {
        self.inner.set_text(text);
    }
}

impl View for BridgedTextViewer {
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
