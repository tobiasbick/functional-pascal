//! `Std.Tui` view, modal, and command binding intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::TuiState;
use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use crate::vm::runtime_error;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::{SolidFillWidget, ViewId, ViewRect, ViewWidget, validate_packed_crt_color};

impl Worker {
    /// Executes `Std.Tui` view, modal, and command binding intrinsics.
    pub(super) fn try_exec_tui_view_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::ApplicationShowModal) => {
                let root_view_id = self.pop_tui_view_id(line)?;
                let root_view_id = self.require_registered_tui_view(root_view_id, line)?;
                let modal_id = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let previous_scope = Self::modal_scope_ids(tui);
                    tui.views.raise(root_view_id);
                    tui.modals.show(fpas_std::ModalId(modal_id), root_view_id);
                    let next_scope = Self::modal_scope_ids(tui);
                    Self::request_scope_redraws(tui, &previous_scope, &next_scope, line);
                    let _ = tui.views.focus_first_in_scope(&next_scope);
                });
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationShowDialog) => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                let modal_id = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let view_rect = ViewRect {
                    x,
                    y,
                    width,
                    height,
                };
                let dialog_root = self.with_tui(|tui| {
                    let previous_scope = Self::modal_scope_ids(tui);
                    let view_id = tui.views.register(view_rect);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    tui.modals.show_dialog(fpas_std::ModalId(modal_id), view_id);
                    let next_scope = Self::modal_scope_ids(tui);
                    Self::request_scope_redraws(tui, &previous_scope, &next_scope, line);
                    let _ = tui.views.focus_first_in_scope(&next_scope);
                    view_id
                });
                self.push(Value::Integer(i64::from(dialog_root.raw())))?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationCloseModal) => {
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    Self::close_active_modal(tui, line);
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostBindCommand) => {
                let command_id = self.pop_int(line)?;
                let key = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    tui.commands.bind(key, fpas_std::CommandId(command_id));
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostBindCommandToView) => {
                let command_id = self.pop_int(line)?;
                let key = self.pop_console_key_event(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.require_registered_tui_view(view_id, line)?;
                self.with_tui(|tui| {
                    tui.view_commands
                        .entry(view_id)
                        .or_default()
                        .bind(key, fpas_std::CommandId(command_id));
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostBindCommandToActiveModal) => {
                let command_id = self.pop_int(line)?;
                let key = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let _ = tui
                        .modals
                        .bind_command_to_active(key, fpas_std::CommandId(command_id));
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostEnterModal) => {
                let modal_id = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    tui.modals.enter(fpas_std::ModalId(modal_id));
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostLeaveModal) => {
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    Self::close_active_modal(tui, line);
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostModalDepth) => {
                self.pop_tui_application(line)?;
                let depth = self.with_tui(|tui| tui.modals.depth() as i64);
                self.push(Value::Integer(depth))?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterView) => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let view_rect = ViewRect {
                    x,
                    y,
                    width,
                    height,
                };
                let view_id = self.with_tui(|tui| {
                    let view_id = tui.views.register(view_rect);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    view_id
                });
                self.push(Value::Integer(i64::from(view_id.raw())))?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostUnregisterView) => {
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    Self::unregister_tui_view_subtree(tui, view_id, line);
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostPushChildView) => {
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.require_registered_tui_view(view_id, line)?;
                self.with_tui(|tui| {
                    let _ = tui.views.push_child(view_id);
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostQueryFocusedViewId) => {
                self.pop_tui_application(line)?;
                let focused_id = self.with_tui(|tui| tui.views.focused_id());
                let packed = focused_id.map_or(-1, |id| i64::from(id.raw()));
                self.push(Value::Integer(packed))?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal) => {
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
            Intrinsic::Tui(TuiIntrinsic::HostSetViewRect) => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let Some(previous_rect) = tui.views.rect(view_id) else {
                        return;
                    };
                    let next_rect = ViewRect {
                        x,
                        y,
                        width,
                        height,
                    };
                    tui.views.set_rect(view_id, next_rect);
                    let _ = tui.session.request_redraw_rect(previous_rect, line);
                    let _ = tui.session.request_redraw_rect(next_rect, line);
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostSetViewParent) => {
                let parent_raw = self.pop_int(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let parent_id = if parent_raw < 0 {
                        None
                    } else {
                        Some(ViewId::from_raw(parent_raw as u32))
                    };
                    let previous_rect = tui.views.rect(view_id);
                    if let Some(parent_id) = parent_id
                        && tui.views.rect(parent_id).is_none()
                    {
                        return;
                    }
                    if tui.views.set_parent(view_id, parent_id) {
                        if let Some(rect) = previous_rect {
                            let _ = tui.session.request_redraw_rect(rect, line);
                        }
                        if let Some(rect) = tui.views.rect(view_id) {
                            let _ = tui.session.request_redraw_rect(rect, line);
                        }
                    }
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnViewPaint) => {
                let func = self.pop(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.require_registered_tui_view(view_id, line)?;
                self.validate_host_handler_function(
                    &func,
                    3,
                    "OnViewPaint",
                    "Pass a `procedure (Application, integer, Std.Tui.Rect)` handler for a registered host view.",
                    line,
                )?;
                self.with_tui(|tui| {
                    tui.view_paints.insert(view_id, func);
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostCreateSolidFillView) => {
                let fill_char = self.pop_optional_char("FillChar", line)?;
                let text_color = self.pop_optional_integer("TextColor", line)?;
                let fill_color = self.pop_int(line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;

                let fill_color = validate_packed_crt_color(fill_color, "FillColor", line)?;
                let text_color = match text_color {
                    None => None,
                    Some(color) => Some(validate_packed_crt_color(color, "TextColor", line)?),
                };

                let view_rect = ViewRect {
                    x,
                    y,
                    width,
                    height,
                };
                let widget = ViewWidget::SolidFill(SolidFillWidget {
                    fill_color,
                    text_color,
                    fill_char,
                });
                let view_id = self.with_tui(|tui| {
                    let view_id = tui.views.register(view_rect);
                    tui.view_widgets.insert(view_id, widget);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    view_id
                });
                self.push(Value::Integer(i64::from(view_id.raw())))?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostCreateMenuBarView) => {
                let style = self.pop_menu_bar_style(line)?;
                let items = self.pop_menu_bar_items(line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;

                let view_rect = ViewRect {
                    x,
                    y,
                    width,
                    height,
                };
                let widget = ViewWidget::MenuBar(fpas_std::MenuBarWidget::new(items, style));
                let view_id = self.with_tui(|tui| {
                    let view_id = tui.views.register(view_rect);
                    tui.view_widgets.insert(view_id, widget);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    view_id
                });
                self.push(Value::Integer(i64::from(view_id.raw())))?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostSetMenuBarItems) => {
                let items = self.pop_menu_bar_items(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                let view_id = self.require_registered_tui_view(view_id, line)?;
                self.with_tui(|tui| {
                    if let Some(ViewWidget::MenuBar(menu)) = tui.view_widgets.get_mut(&view_id) {
                        menu.set_items(items);
                        if let Some(rect) = tui.views.rect(view_id) {
                            let _ = tui.session.request_redraw_rect(rect, line);
                        }
                    }
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostCreateStatusBarView) => {
                let style = self.pop_status_bar_style(line)?;
                let segments = self.pop_status_bar_segments(line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;

                let view_rect = ViewRect {
                    x,
                    y,
                    width,
                    height,
                };
                let widget = ViewWidget::StatusBar(fpas_std::StatusBarWidget::new(segments, style));
                let view_id = self.with_tui(|tui| {
                    let view_id = tui.views.register(view_rect);
                    tui.view_widgets.insert(view_id, widget);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    view_id
                });
                self.push(Value::Integer(i64::from(view_id.raw())))?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostSetStatusBarSegments) => {
                let segments = self.pop_status_bar_segments(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                let view_id = self.require_registered_tui_view(view_id, line)?;
                self.with_tui(|tui| {
                    if let Some(ViewWidget::StatusBar(status)) = tui.view_widgets.get_mut(&view_id)
                    {
                        status.set_segments(segments);
                        if let Some(rect) = tui.views.rect(view_id) {
                            let _ = tui.session.request_redraw_rect(rect, line);
                        }
                    }
                });
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    pub(in crate::vm::execute::io) fn pop_tui_view_id(
        &mut self,
        line: SourceLocation,
    ) -> Result<ViewId, VmError> {
        let raw = self.pop_int(line)?;
        let raw = u32::try_from(raw).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("ViewId {raw} is out of range (expected 0..={})", u32::MAX),
                "Pass the integer handle returned by `Application.HostRegisterView(App, X, Y, Width, Height)`.",
                line,
            )
        })?;
        Ok(ViewId::from_raw(raw))
    }

    pub(in crate::vm::execute::io) fn require_registered_tui_view(
        &self,
        view_id: ViewId,
        line: SourceLocation,
    ) -> Result<ViewId, VmError> {
        let exists = self.with_tui(|tui| tui.views.rect(view_id).is_some());
        if exists {
            Ok(view_id)
        } else {
            Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Unknown host view handle {}", view_id.raw()),
                "Pass a view id returned by `Application.HostRegisterView(App, X, Y, Width, Height)`.",
                line,
            ))
        }
    }

    /// Returns the view ids that belong to the currently active modal scope.
    pub(super) fn modal_scope_ids(tui: &TuiState) -> Vec<ViewId> {
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

    fn unregister_tui_view_subtree(tui: &mut TuiState, root_view: ViewId, line: SourceLocation) {
        let subtree = tui.views.subtree_ids(root_view);
        if subtree.is_empty() {
            return;
        }

        let previous_focus = tui.views.focused_id();
        Self::request_scope_redraws(tui, &subtree, &[], line);
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
