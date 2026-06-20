//! View-tree registration, geometry, parenting, and local paint handlers.

use crate::vm::diagnostics::VmError;
use crate::vm::{TuiState, Worker};
use fpas_bytecode::{SourceLocation, TuiIntrinsic};
use fpas_std::{ViewId, ViewRect};

use super::super::view_geometry::validate_view_rect;

impl Worker {
    /// Executes view-tree registration, geometry, parenting, and paint-handler intrinsics.
    pub(super) fn try_exec_tui_view_tree_intrinsic(
        &mut self,
        intrinsic: TuiIntrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            TuiIntrinsic::HostRegisterView => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let view_rect = validate_view_rect(
                    "Application.HostRegisterView",
                    ViewRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    line,
                )?;
                let view_id = self.with_tui(|tui| {
                    let view_id = tui.views.register(view_rect);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    view_id
                });
                self.push(Self::tui_view_id_record(view_id))?;
            }
            TuiIntrinsic::HostUnregisterView => {
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| Self::unregister_tui_view_subtree(tui, view_id, line));
            }
            TuiIntrinsic::HostPushChildView => {
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.require_registered_tui_view(view_id, line)?;
                self.with_tui(|tui| {
                    let _ = tui.views.push_child(view_id);
                });
            }
            TuiIntrinsic::HostSetViewRect => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                let next_rect = validate_view_rect(
                    "Application.HostSetViewRect",
                    ViewRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    line,
                )?;
                self.with_tui(|tui| {
                    if tui.views.rect(view_id).is_none() {
                        return;
                    }
                    let previous_rects = Self::subtree_screen_rects(tui, view_id);
                    tui.views.set_rect(view_id, next_rect);
                    let next_rects = Self::subtree_screen_rects(tui, view_id);
                    Self::request_rect_redraws(tui, &previous_rects, &next_rects, line);
                });
            }
            TuiIntrinsic::HostSetViewParent => {
                let parent = self.pop_optional_tui_view_id("Parent", line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    if let Some(parent_id) = parent
                        && tui.views.rect(parent_id).is_none()
                    {
                        return;
                    }
                    let previous_rects = Self::subtree_screen_rects(tui, view_id);
                    if tui.views.set_parent(view_id, parent) {
                        let next_rects = Self::subtree_screen_rects(tui, view_id);
                        Self::request_rect_redraws(tui, &previous_rects, &next_rects, line);
                    }
                });
            }
            TuiIntrinsic::HostRegisterOnViewPaint => {
                let func = self.pop(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.require_registered_tui_view(view_id, line)?;
                self.validate_host_handler_function(
                    &func,
                    3,
                    "OnViewPaint",
                    "Pass a `procedure (Application, ViewId, Std.Tui.Rect)` handler for a registered host view.",
                    line,
                )?;
                self.with_tui(|tui| {
                    tui.view_paints.insert(view_id, func);
                });
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    /// Removes a view subtree and all VM-owned state associated with its handles.
    pub(super) fn unregister_tui_view_subtree(
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

    fn subtree_screen_rects(tui: &TuiState, root: ViewId) -> Vec<ViewRect> {
        tui.views
            .subtree_ids(root)
            .into_iter()
            .filter_map(|id| tui.views.rect(id))
            .collect()
    }

    fn request_rect_redraws(
        tui: &mut TuiState,
        previous: &[ViewRect],
        next: &[ViewRect],
        line: SourceLocation,
    ) {
        for rect in previous.iter().chain(next) {
            let _ = tui.session.request_redraw_rect(*rect, line);
        }
    }
}
