//! Modal view lifecycle and active-scope tracking.
//!
//! **Documentation:** `docs/pascal/std/tui/app/modals.md`

use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::{TuiState, Worker};
use fpas_bytecode::{SourceLocation, TuiIntrinsic};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::{ModalClose, ModalId, ModalResult, ViewId, ViewRect};

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
                    let previous_active_root = tui.views.active_root();
                    let previous_focus = tui.views.focused_id();
                    tui.views.raise(root_view_id);
                    tui.modals.show(ModalId(modal_id), root_view_id);
                    tui.modals
                        .set_return_context(previous_active_root, previous_focus);
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
                    let previous_active_root = tui.views.active_root();
                    let previous_focus = tui.views.focused_id();
                    let view_id = tui.views.register(view_rect);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    tui.modals.show_dialog(ModalId(modal_id), view_id);
                    tui.modals
                        .set_return_context(previous_active_root, previous_focus);
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
                    let previous_active_root = tui.views.active_root();
                    let previous_focus = tui.views.focused_id();
                    tui.modals.enter(ModalId(modal_id));
                    tui.modals
                        .set_return_context(previous_active_root, previous_focus);
                });
            }
            TuiIntrinsic::HostSetActiveModalResult => {
                let result_code = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let result = Self::validate_modal_result_code(result_code, line)?;
                let set = self.with_tui(|tui| tui.modals.set_result(result));
                if !set {
                    return Err(runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "Application.HostSetActiveModalResult requires an active modal frame",
                        "Open a modal with `Application.ShowModal`, `Application.ShowDialog`, or `Application.HostEnterModal` before setting its result.",
                        line,
                    ));
                }
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

    fn validate_modal_result_code(
        result_code: i64,
        line: SourceLocation,
    ) -> Result<ModalResult, VmError> {
        match result_code {
            1 => Ok(ModalResult::Accept),
            2 => Ok(ModalResult::Cancel),
            code if code >= 1000 => Ok(ModalResult::Command(code)),
            _ => Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!(
                    "Application.HostSetActiveModalResult expects 1 (Accept), 2 (Cancel), or an application-defined result code >= 1000, got {result_code}"
                ),
                "Use 1 for Accept, 2 for Cancel, or reserve command-like dialog results at 1000 and above.",
                line,
            )),
        }
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
