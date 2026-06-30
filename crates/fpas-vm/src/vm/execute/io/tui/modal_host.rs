//! Modal scope helpers for the transitional retained host loop.
//!
//! **Documentation:** `docs/pascal/std/tui/app/modals.md`

use crate::vm::{TuiState, Worker};
use fpas_bytecode::SourceLocation;
use fpas_std::{ModalClose, ViewId};

impl Worker {
    /// Returns the view ids that belong to the currently active modal scope.
    pub(in crate::vm::execute::io::tui) fn modal_scope_ids(tui: &TuiState) -> Vec<ViewId> {
        let mut scope = tui
            .modals
            .active_root_view()
            .map(|root| tui.views.subtree_ids(root))
            .unwrap_or_default();

        if let Some(extra_views) = tui.modals.active_scoped_views() {
            for view_id in extra_views {
                if !scope.contains(view_id) {
                    scope.push(*view_id);
                }
            }
        }

        scope
    }

    pub(in crate::vm::execute::io::tui) fn close_active_modal(
        tui: &mut TuiState,
        line: SourceLocation,
    ) {
        let previous_scope = Self::modal_scope_ids(tui);
        let close = tui.modals.leave_with_context();
        let next_scope = Self::modal_scope_ids(tui);
        Self::request_scope_redraws(tui, &previous_scope, &next_scope, line);

        if let Some(close) = close {
            if close.owns_root_view
                && let Some(root_view) = close.root_view
            {
                Self::unregister_tui_view_subtree(tui, root_view, line);
            }
            Self::restore_modal_return_focus(tui, &close, &next_scope, line);
        }
    }

    /// Removes a view subtree and all VM-owned state associated with its handles.
    pub(in crate::vm::execute::io::tui) fn unregister_tui_view_subtree(
        tui: &mut TuiState,
        root_view: ViewId,
        line: SourceLocation,
    ) {
        let subtree = tui.views.subtree_ids(root_view);
        if subtree.is_empty() {
            return;
        }

        let previous_focus = tui.views.focused_id();
        for view_id in &subtree {
            if let Some(rect) = tui.views.rect(*view_id) {
                let _ = tui.session.request_redraw_rect(rect, line);
            }
        }
        Self::clear_view_local_state(tui, &subtree);
        tui.modals.remove_view_references(&subtree);
        tui.views.unregister(root_view);

        let current_focus = tui.views.focused_id();
        if current_focus != previous_focus
            && let Some(view_id) = current_focus
            && let Some(rect) = tui.views.rect(view_id)
        {
            let _ = tui.session.request_redraw_rect(rect, line);
        }
    }

    fn clear_view_local_state(tui: &mut TuiState, view_ids: &[ViewId]) {
        for view_id in view_ids {
            tui.view_paints.remove(view_id);
            tui.view_widgets.remove(view_id);
            tui.view_commands.remove(view_id);
        }
    }

    fn restore_modal_return_focus(
        tui: &mut TuiState,
        close: &ModalClose,
        next_scope: &[ViewId],
        line: SourceLocation,
    ) {
        let previous_focus = tui.views.focused_id();
        let restored_focus = close
            .previous_focus
            .is_some_and(|view_id| tui.views.focus_view(view_id).0);

        if !restored_focus {
            let restored_root = close
                .previous_active_root
                .is_some_and(|root| Self::restore_modal_return_root(tui, root));
            if !restored_root && !next_scope.is_empty() {
                let _ = tui.views.focus_first_in_scope(next_scope);
            } else if !restored_root {
                let _ = tui.views.focus_next();
            }
        }

        let current_focus = tui.views.focused_id();
        Self::request_focus_redraws(tui, previous_focus, current_focus, line);
    }

    fn restore_modal_return_root(tui: &mut TuiState, root: ViewId) -> bool {
        let previous_focus = tui.views.focused_id();
        let Some(_) = tui.views.activate_root(root) else {
            return false;
        };
        tui.views.focused_id() != previous_focus || tui.views.active_root() == Some(root)
    }

    fn request_focus_redraws(
        tui: &mut TuiState,
        previous_focus: Option<ViewId>,
        current_focus: Option<ViewId>,
        line: SourceLocation,
    ) {
        for view_id in [previous_focus, current_focus].into_iter().flatten() {
            if let Some(rect) = tui.views.rect(view_id) {
                let _ = tui.session.request_redraw_rect(rect, line);
            }
            if let Some(rect) = tui
                .views
                .root_of(view_id)
                .and_then(|root| tui.views.rect(root))
            {
                let _ = tui.session.request_redraw_rect(rect, line);
            }
        }
    }

    fn request_scope_redraws(
        tui: &mut TuiState,
        previous_scope: &[ViewId],
        next_scope: &[ViewId],
        line: SourceLocation,
    ) {
        for view_id in previous_scope.iter().chain(next_scope.iter()) {
            if let Some(rect) = tui.views.rect(*view_id) {
                let _ = tui.session.request_redraw_rect(rect, line);
            }
        }
    }
}
