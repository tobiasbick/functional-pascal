//! View-subtree and frame-root redraw damage helpers.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::vm::{TuiState, Worker};
use fpas_bytecode::SourceLocation;
use fpas_std::{ViewId, ViewRect};

impl Worker {
    /// Resolved screen rectangles for every view in `root`'s subtree.
    pub(in crate::vm::execute::io::tui) fn subtree_screen_rects(
        tui: &TuiState,
        root: ViewId,
    ) -> Vec<ViewRect> {
        tui.views
            .subtree_ids(root)
            .into_iter()
            .filter_map(|id| tui.views.rect(id))
            .collect()
    }

    /// Subtree screen rects plus optional frame-root shadow regions.
    pub(in crate::vm::execute::io::tui) fn frame_damage_rects(
        tui: &TuiState,
        root: ViewId,
    ) -> Vec<ViewRect> {
        let mut rects = Self::subtree_screen_rects(tui, root);
        if let Some(shadow) = tui.views.root_shadow(root) {
            for rect in [shadow.right, shadow.bottom].into_iter().flatten() {
                if rect.width > 0 && rect.height > 0 {
                    rects.push(rect);
                }
            }
        }
        rects
    }

    /// Marks every rectangle from the previous and next layout snapshots dirty.
    pub(in crate::vm::execute::io::tui) fn request_rect_redraws(
        tui: &mut TuiState,
        previous: &[ViewRect],
        next: &[ViewRect],
        line: SourceLocation,
    ) {
        for rect in previous.iter().chain(next) {
            let _ = tui.session.request_redraw_rect(*rect, line);
        }
    }

    /// Marks frame-root subtree damage, optionally pairing a pre-move snapshot with the current layout.
    pub(in crate::vm::execute::io::tui) fn request_frame_subtree_damage(
        tui: &mut TuiState,
        previous: Option<&[ViewRect]>,
        root: ViewId,
        line: SourceLocation,
    ) {
        let next = Self::frame_damage_rects(tui, root);
        match previous {
            None => Self::request_rect_redraws(tui, &next, &[], line),
            Some(prev) => Self::request_rect_redraws(tui, prev, &next, line),
        }
    }

    /// Marks damage for every registered root's subtree before and after a bulk layout operation.
    pub(in crate::vm::execute::io::tui) fn request_all_roots_subtree_damage(
        tui: &mut TuiState,
        previous: &[ViewRect],
        line: SourceLocation,
    ) {
        let next = tui
            .views
            .roots()
            .iter()
            .flat_map(|root| Self::frame_damage_rects(tui, *root))
            .collect::<Vec<_>>();
        Self::request_rect_redraws(tui, previous, &next, line);
    }

    /// Collects frame damage rects for every registered root.
    pub(in crate::vm::execute::io::tui) fn all_roots_damage_rects(tui: &TuiState) -> Vec<ViewRect> {
        tui.views
            .roots()
            .iter()
            .flat_map(|root| Self::frame_damage_rects(tui, *root))
            .collect()
    }
}
