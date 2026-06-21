//! Frame-root host intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, TuiIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::{
    FrameCapabilities, FrameContentSize, FrameGeometryError, FrameKind, FrameRootSpec,
    FrameRootState, ViewRect,
};

use super::super::view_geometry::validate_view_rect;

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
            TuiIntrinsic::HostCreateFrameRootView => {
                let scrollable = self.pop_bool(line)?;
                let zoomable = self.pop_bool(line)?;
                let resizable = self.pop_bool(line)?;
                let movable = self.pop_bool(line)?;
                let kind = self.pop_frame_kind(line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let outer = validate_view_rect(
                    "Application.HostCreateFrameRootView",
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
                        closable: false,
                        scrollable,
                    },
                    options: Default::default(),
                };
                let view_id = self.with_tui(|tui| {
                    tui.views
                        .register_frame_root(spec)
                        .map(|frame| frame.view_id)
                        .map_err(|error| frame_geometry_error(error, line))
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
                        "Pass a handle returned by Application.HostCreateFrameRootView.",
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

fn frame_geometry_error(error: FrameGeometryError, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "frame geometry requires at least {}x{} cells, got {}x{}",
            error.min_width, error.min_height, error.got_width, error.got_height
        ),
        "Increase the requested width and height or disable scrollable frame chrome.",
        line,
    )
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
                "zoomed".into(),
                Value::Boolean(state.pre_zoom_rect.is_some()),
            ),
        ],
    }
}
