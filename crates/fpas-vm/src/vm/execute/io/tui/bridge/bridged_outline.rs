//! Turbo Vision bridge modal `OutlineViewer` wired to FPAS outline selection state.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::vm::shared::TurboVisionOutlineNode;
use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{LISTBOX_FOCUSED, LISTBOX_NORMAL, LISTBOX_SELECTED, Palette};
use turbo_vision::core::palette_chain::PaletteChainNode;
use turbo_vision::core::state::{GrowFlags, StateFlags};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::list_viewer::ListViewer;
use turbo_vision::views::outline::OutlineViewer;
use turbo_vision::views::view::{View, write_line_to_terminal};

use super::outline_nodes::build_outline_tv_roots;

/// Turbo Vision outline viewer wired to a shared FPAS selection cell.
pub(in crate::vm::execute::io::tui) struct BridgedOutline {
    inner: OutlineViewer<String>,
    selection_cell: TurboVisionListSelectionCell,
    /// Grow flags used when the outline is nested in a resizable parent.
    grow_mode: GrowFlags,
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
            grow_mode: 0,
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
        // Upstream `OutlineViewer::draw` uses fixed listbox attributes and ignores the
        // owner palette chain. Remap through `map_color` so outlines inside blue windows
        // inherit window colors instead of painting a light-gray background.
        let bounds = self.bounds();
        let width = bounds.width_clamped() as usize;
        let height = bounds.height_clamped() as usize;
        let list_state = self.inner.list_state();
        let item_count = self.inner.item_count();

        let color_normal = if self.is_focused() {
            self.map_color(LISTBOX_FOCUSED)
        } else {
            self.map_color(LISTBOX_NORMAL)
        };
        let color_selected = self.map_color(LISTBOX_SELECTED);

        for row in 0..height {
            let mut buf = DrawBuffer::new(width);
            let item_idx = list_state.top_item + row;

            if item_idx < item_count {
                let is_selected = list_state.focused == Some(item_idx);
                let color = if is_selected {
                    color_selected
                } else {
                    color_normal
                };
                let text = self.inner.get_text(item_idx, width);
                buf.move_str(0, &text, color);
                let text_len = text.chars().count();
                if text_len < width {
                    buf.move_char(text_len, ' ', color, width - text_len);
                }
            } else {
                buf.move_char(0, ' ', color_normal, width);
            }

            write_line_to_terminal(terminal, bounds.a.x, bounds.a.y + row as i16, &buf);
        }
    }

    fn grow_mode(&self) -> GrowFlags {
        self.grow_mode
    }

    fn set_grow_mode(&mut self, grow_mode: GrowFlags) {
        self.grow_mode = grow_mode;
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
    fn grow_mode_stretches_with_parent_group() {
        use turbo_vision::core::state::{GF_GROW_HI_X, GF_GROW_HI_Y};
        use turbo_vision::views::group::Group;

        let selection_cell = TurboVisionListSelectionCell::new(Some(0));
        let mut outline =
            BridgedOutline::new(Rect::new(0, 0, 10, 5), &sample_roots(), selection_cell);
        outline.set_grow_mode(GF_GROW_HI_X | GF_GROW_HI_Y);

        let mut group = Group::new(Rect::new(0, 0, 10, 5));
        group.add(Box::new(outline));
        group.set_bounds(Rect::new(0, 0, 20, 10));

        let child = group.child_at(0);
        assert_eq!(child.bounds().width(), 20);
        assert_eq!(child.bounds().height(), 10);
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
