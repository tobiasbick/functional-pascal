//! Modal view lifecycle and active-scope tracking.

use crate::vm::diagnostics::VmError;
use crate::vm::{TuiState, Worker};
use fpas_bytecode::{SourceLocation, TuiIntrinsic};
use fpas_std::{ModalId, ViewId, ViewRect};

use super::super::view_geometry::validate_view_rect;

impl Worker {
    /// Executes modal lifecycle and modal-scope intrinsics.
    pub(super) fn try_exec_tui_modal_intrinsic(
        &mut self,
        intrinsic: TuiIntrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            TuiIntrinsic::ApplicationShowModal => {
                let root_view_id = self.pop_tui_view_id(line)?;
                let root_view_id = self.require_registered_tui_view(root_view_id, line)?;
                let modal_id = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let previous_scope = Self::modal_scope_ids(tui);
                    tui.views.raise(root_view_id);
                    tui.modals.show(ModalId(modal_id), root_view_id);
                    let next_scope = Self::modal_scope_ids(tui);
                    Self::request_scope_redraws(tui, &previous_scope, &next_scope, line);
                    let _ = tui.views.focus_first_in_scope(&next_scope);
                });
            }
            TuiIntrinsic::ApplicationShowDialog => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                let modal_id = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let view_rect = validate_view_rect(
                    "Application.ShowDialog",
                    ViewRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    line,
                )?;
                let dialog_root = self.with_tui(|tui| {
                    let previous_scope = Self::modal_scope_ids(tui);
                    let view_id = tui.views.register(view_rect);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    tui.modals.show_dialog(ModalId(modal_id), view_id);
                    let next_scope = Self::modal_scope_ids(tui);
                    Self::request_scope_redraws(tui, &previous_scope, &next_scope, line);
                    let _ = tui.views.focus_first_in_scope(&next_scope);
                    view_id
                });
                self.push(Self::tui_view_id_record(dialog_root))?;
            }
            TuiIntrinsic::ApplicationCloseModal | TuiIntrinsic::HostLeaveModal => {
                self.pop_tui_application(line)?;
                self.with_tui(|tui| Self::close_active_modal(tui, line));
            }
            TuiIntrinsic::HostEnterModal => {
                let modal_id = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    tui.modals.enter(ModalId(modal_id));
                });
            }
            TuiIntrinsic::HostAttachViewToActiveModal => {
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.require_registered_tui_view(view_id, line)?;
                self.with_tui(|tui| {
                    if tui.modals.attach_view_to_active(view_id)
                        && let Some(rect) = tui.views.rect(view_id)
                    {
                        let _ = tui.session.request_redraw_rect(rect, line);
                    }
                });
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

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

    fn close_active_modal(tui: &mut TuiState, line: SourceLocation) {
        let previous_scope = Self::modal_scope_ids(tui);
        let popped = tui.modals.leave_with_scope_info();
        let next_scope = Self::modal_scope_ids(tui);
        Self::request_scope_redraws(tui, &previous_scope, &next_scope, line);

        if let Some((_, Some(root_view), true, _)) = popped {
            Self::unregister_tui_view_subtree(tui, root_view, line);
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
