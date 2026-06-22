//! Frame-root host intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, TuiIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::{
    FrameCapabilities, FrameContentSize, FrameKind, FrameRootSpec, FrameRootState,
    FrameScrollState, FrameWidget, ViewRect, ViewWidget,
};

use super::super::view_geometry::validate_view_rect;
use super::frame_geometry_error;

impl Worker {
    pub(in crate::vm::execute::io::tui) fn try_exec_tui_frame_tui_intrinsic(
        &mut self,
        intrinsic: TuiIntrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            TuiIntrinsic::HostSetDesktopWorkArea => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let rect = validate_view_rect(
                    "Application.HostSetDesktopWorkArea",
                    ViewRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    line,
                )?;
                let ok = self.with_tui(|tui| tui.views.set_desktop_work_area(rect));
                self.stack.push(Value::Boolean(ok));
            }
            TuiIntrinsic::HostCreateFrameView => {
                let closable = self.pop_bool(line)?;
                let scrollable = self.pop_bool(line)?;
                let zoomable = self.pop_bool(line)?;
                let resizable = self.pop_bool(line)?;
                let movable = self.pop_bool(line)?;
                let kind = self.pop_frame_kind(line)?;
                let title = self.pop_control_string("Title", line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let outer = validate_view_rect(
                    "Application.HostCreateFrameView",
                    ViewRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    line,
                )?;
                let spec = FrameRootSpec {
                    kind,
                    outer,
                    content_size: FrameContentSize::new(0, 0),
                    capabilities: FrameCapabilities {
                        movable,
                        resizable,
                        zoomable,
                        closable,
                        scrollable,
                    },
                    options: Default::default(),
                };
                let view_id = self.with_tui(|tui| {
                    let frame = tui
                        .views
                        .register_frame_root(spec)
                        .map_err(|error| frame_geometry_error(error, line))?;
                    tui.view_widgets.insert(
                        frame.view_id,
                        ViewWidget::Frame(FrameWidget::new(
                            title,
                            kind,
                            spec.capabilities,
                            spec.content_size,
                        )),
                    );
                    let _ = tui.session.request_redraw_rect(frame.geometry.outer, line);
                    Ok(frame.view_id)
                })?;
                self.push(Self::tui_view_id_record(view_id))?;
            }
            TuiIntrinsic::HostActivateNextWindow => {
                self.pop_tui_application(line)?;
                let changed = self.with_tui(|tui| {
                    let exclude = tui
                        .modals
                        .active_root_view()
                        .into_iter()
                        .collect::<Vec<_>>();
                    tui.views.activate_next_root_excluding(&exclude).is_some()
                });
                self.stack.push(Value::Boolean(changed));
            }
            TuiIntrinsic::HostZoomFrameRoot => {
                let id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                let ok = self.with_tui(|tui| tui.views.zoom_frame_root(id));
                self.stack.push(Value::Boolean(ok));
            }
            TuiIntrinsic::HostRestoreFrameRoot => {
                let id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                let ok = self.with_tui(|tui| tui.views.restore_frame_root(id));
                self.stack.push(Value::Boolean(ok));
            }
            TuiIntrinsic::QueryFrameRootState => {
                let id = self.pop_query_view_id(line)?;
                self.pop_tui_application(line)?;
                let value = self.with_tui(|tui| {
                    let state = tui.views.frame_root_state(id)?;
                    let rect = tui.views.rect(id)?;
                    Some(frame_root_record(*state, rect))
                });
                let Some(record) = value else {
                    return Err(runtime_error(
                        RUNTIME_CONSOLE_STATE_ERROR,
                        "ViewId is not a registered frame root",
                        "Pass a handle returned by Application.HostCreateFrameView.",
                        line,
                    ));
                };
                self.stack.push(record);
            }
            TuiIntrinsic::HostCascadeFrameRoots => {
                let step_y = self.pop_int(line)?;
                let step_x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let count = self.with_tui(|tui| {
                    let exclude = tui
                        .modals
                        .active_root_view()
                        .into_iter()
                        .collect::<Vec<_>>();
                    tui.views
                        .cascade_frame_roots_excluding(&exclude, step_x, step_y)
                });
                self.stack.push(Value::Integer(count as i64));
            }
            TuiIntrinsic::HostTileFrameRoots => {
                self.pop_tui_application(line)?;
                let count = self.with_tui(|tui| {
                    let exclude = tui
                        .modals
                        .active_root_view()
                        .into_iter()
                        .collect::<Vec<_>>();
                    tui.views.tile_frame_roots_excluding(&exclude)
                });
                self.stack.push(Value::Integer(count as i64));
            }
            TuiIntrinsic::HostSetFrameContentSize => {
                let content_height = self.pop_int(line)?;
                let content_width = self.pop_int(line)?;
                let id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                let ok = self.with_tui(|tui| {
                    tui.views
                        .set_frame_content_size(id, content_width, content_height)
                });
                if ok {
                    self.sync_frame_widget_scroll(id);
                    let _ = self.with_tui(|tui| {
                        tui.views
                            .rect(id)
                            .and_then(|rect| tui.session.request_redraw_rect(rect, line).ok())
                    });
                }
            }
            TuiIntrinsic::HostScrollFrame => {
                let delta_y = self.pop_int(line)?;
                let delta_x = self.pop_int(line)?;
                let id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                let changed = self.with_tui(|tui| tui.views.scroll_frame(id, delta_x, delta_y));
                if changed {
                    self.sync_frame_widget_scroll(id);
                    let _ = self.with_tui(|tui| {
                        tui.views
                            .rect(id)
                            .and_then(|rect| tui.session.request_redraw_rect(rect, line).ok())
                    });
                }
            }
            TuiIntrinsic::HostSetFrameScrollOffset => {
                let offset_y = self.pop_int(line)?;
                let offset_x = self.pop_int(line)?;
                let id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                let changed =
                    self.with_tui(|tui| tui.views.set_frame_scroll_offset(id, offset_x, offset_y));
                if changed {
                    self.sync_frame_widget_scroll(id);
                    let _ = self.with_tui(|tui| {
                        tui.views
                            .rect(id)
                            .and_then(|rect| tui.session.request_redraw_rect(rect, line).ok())
                    });
                }
            }
            TuiIntrinsic::QueryFrameScrollState => {
                let id = self.pop_query_view_id(line)?;
                self.pop_tui_application(line)?;
                let value = self.with_tui(|tui| {
                    let state = tui.views.frame_scroll_state(id)?;
                    Some(frame_scroll_record(state))
                });
                let Some(record) = value else {
                    return Err(runtime_error(
                        RUNTIME_CONSOLE_STATE_ERROR,
                        "ViewId is not a registered frame root",
                        "Pass a handle returned by Application.HostCreateFrameView.",
                        line,
                    ));
                };
                self.stack.push(record);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn pop_frame_kind(&mut self, line: SourceLocation) -> Result<FrameKind, VmError> {
        let value = self.pop_int(line)?;
        match value {
            0 => Ok(FrameKind::Window),
            1 => Ok(FrameKind::Dialog),
            other => Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Frame kind must be 0 (Window) or 1 (Dialog), got {other}"),
                "Pass 0 for Window or 1 for Dialog.",
                line,
            )),
        }
    }
}

fn frame_scroll_record(state: FrameScrollState) -> Value {
    Value::Record {
        type_name: "Std.Tui.FrameScrollState".into(),
        fields: vec![
            ("offsetX".into(), Value::Integer(state.offset_x)),
            ("offsetY".into(), Value::Integer(state.offset_y)),
            ("contentWidth".into(), Value::Integer(state.content_width)),
            ("contentHeight".into(), Value::Integer(state.content_height)),
        ],
    }
}

fn frame_root_record(state: FrameRootState, rect: ViewRect) -> Value {
    Value::Record {
        type_name: "Std.Tui.FrameRootState".into(),
        fields: vec![
            ("x".into(), Value::Integer(rect.x)),
            ("y".into(), Value::Integer(rect.y)),
            ("width".into(), Value::Integer(rect.width)),
            ("height".into(), Value::Integer(rect.height)),
            (
                "kind".into(),
                Value::Integer(match state.kind {
                    FrameKind::Window => 0,
                    FrameKind::Dialog => 1,
                }),
            ),
            ("movable".into(), Value::Boolean(state.capabilities.movable)),
            (
                "resizable".into(),
                Value::Boolean(state.capabilities.resizable),
            ),
            (
                "zoomable".into(),
                Value::Boolean(state.capabilities.zoomable),
            ),
            (
                "scrollable".into(),
                Value::Boolean(state.capabilities.scrollable),
            ),
            (
                "closable".into(),
                Value::Boolean(state.capabilities.closable),
            ),
            (
                "zoomed".into(),
                Value::Boolean(state.pre_zoom_rect.is_some()),
            ),
        ],
    }
}
