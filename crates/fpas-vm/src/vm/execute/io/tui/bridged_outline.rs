//! Modal `OutlineViewer` wired to FPAS outline selection state.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::vm::shared::TurboVisionOutlineNode;
use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Palette;
use turbo_vision::core::palette_chain::PaletteChainNode;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::list_viewer::ListViewer;
use turbo_vision::views::outline::OutlineViewer;
use turbo_vision::views::view::View;

use super::try2::outline_nodes::build_outline_tv_roots;

/// Turbo Vision outline viewer wired to a shared FPAS selection cell.
pub(in crate::vm::execute::io::tui) struct BridgedOutline {
    inner: OutlineViewer<String>,
    selection_cell: TurboVisionListSelectionCell,
}

impl BridgedOutline {
    /// Build an outline viewer from FPAS node data and a selection cell.
    pub fn new(
        bounds: Rect,
        roots: &[TurboVisionOutlineNode],
        selection_cell: TurboVisionListSelectionCell,
    ) -> Self {
        let mut inner = OutlineViewer::new(bounds, |text: &String| text.clone());
        for root in build_outline_tv_roots(roots) {
            inner.add_root(root);
        }
        if let Some(selection) = selection_cell.read() {
            inner.set_list_selection(selection);
        }
        let mut bridged = Self {
            inner,
            selection_cell,
        };
        bridged.sync_selection();
        bridged
    }

    fn sync_selection(&mut self) {
        self.selection_cell.set(self.inner.list_state().focused);
    }

    /// Replace outline roots from FPAS host state (live patch path).
    pub(in crate::vm::execute::io::tui) fn set_roots_from_fpas(
        &mut self,
        roots: Vec<TurboVisionOutlineNode>,
        selection: Option<usize>,
    ) {
        let tv_roots = build_outline_tv_roots(&roots);
        self.inner.set_roots(tv_roots);
        if let Some(index) = selection {
            self.inner.set_list_selection(index);
        }
        self.sync_selection();
    }

    /// Copies the upstream outline selection into the shared FPAS cell.
    pub(in crate::vm::execute::io::tui) fn sync_selection_from_view(&mut self) {
        self.sync_selection();
    }
}

impl View for BridgedOutline {
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
    use crate::vm::shared::TurboVisionOutlineNode;
    use turbo_vision::core::event::{Event, KB_DOWN, KB_RIGHT};

    fn sample_roots() -> Vec<TurboVisionOutlineNode> {
        vec![TurboVisionOutlineNode {
            text: "root".into(),
            expanded: false,
            children: vec![TurboVisionOutlineNode {
                text: "child".into(),
                expanded: false,
                children: Vec::new(),
            }],
        }]
    }

    #[test]
    fn keyboard_navigation_syncs_selection_cell() {
        let selection_cell = TurboVisionListSelectionCell::new(Some(0));
        let mut outline = BridgedOutline::new(
            Rect::new(0, 0, 20, 4),
            &sample_roots(),
            selection_cell.clone(),
        );

        assert_eq!(selection_cell.read(), Some(0));

        let mut expand = Event::keyboard(KB_RIGHT);
        outline.handle_event(&mut expand);
        assert_eq!(selection_cell.read(), Some(0));

        let mut down = Event::keyboard(KB_DOWN);
        outline.handle_event(&mut down);
        assert_eq!(selection_cell.read(), Some(1));
    }
}
