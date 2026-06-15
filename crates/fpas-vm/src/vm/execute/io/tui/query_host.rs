//! Native TUI query intrinsics (Phase 3–4).
//!
//! **Documentation:** `docs/pascal/std/tui-app.md`, `docs/future/tui-tests-fpas/README.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::ViewId;

impl Worker {
    /// Executes read-only native TUI query intrinsics.
    pub(super) fn try_exec_tui_query_host_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::QueryScreenSize) => {
                self.pop_tui_application(line)?;
                let (width, height) =
                    self.with_console(|console| (console.screen_width(), console.screen_height()));
                self.push(Self::tui_size_record(width, height))?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryScreenLine) => {
                let y = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let y = Self::screen_row_to_u16(y, line)?;
                let row = self.with_console(|console| console.query_screen_line(y));
                self.push(Value::Str(row))?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryScreenCell) => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let x = Self::screen_column_to_u16(x, line)?;
                let y = Self::screen_row_to_u16(y, line)?;
                let (ch, fg, bg) = self.with_console(|console| {
                    console
                        .query_screen_cell(x, y)
                        .ok_or_else(|| query_cell_error(x, y, line))
                })?;
                self.push(Self::tui_screen_cell_record(ch, fg, bg))?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryRootViews) => {
                self.pop_tui_application(line)?;
                let ids = self.with_tui(|tui| {
                    tui.views
                        .roots()
                        .iter()
                        .map(|id| Value::Integer(i64::from(id.raw())))
                        .collect::<Vec<_>>()
                });
                self.push(Value::Array(ids))?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryViewRect) => {
                let view_id = self.pop_query_view_id(line)?;
                self.pop_tui_application(line)?;
                let rect = self
                    .with_tui(|tui| tui.views.rect(view_id))
                    .ok_or_else(|| query_view_rect_error(view_id, line))?;
                self.push(Self::tui_rect_record(rect))?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryViewParent) => {
                let view_id = self.pop_query_view_id(line)?;
                self.pop_tui_application(line)?;
                let parent = self.with_tui(|tui| tui.views.parent(view_id));
                self.push(match parent {
                    Some(id) => Value::OptionSome(Box::new(Value::Integer(i64::from(id.raw())))),
                    None => Value::OptionNone,
                })?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryViewChildren) => {
                let view_id = self.pop_query_view_id(line)?;
                self.pop_tui_application(line)?;
                let children = self.with_tui(|tui| {
                    tui.views
                        .children(view_id)
                        .iter()
                        .map(|id| Value::Integer(i64::from(id.raw())))
                        .collect::<Vec<_>>()
                });
                self.push(Value::Array(children))?;
            }
            Intrinsic::Tui(TuiIntrinsic::QueryMenuBarState) => {
                let view_id = self.pop_query_view_id(line)?;
                self.pop_tui_application(line)?;
                let snapshot = self.with_tui(|tui| match tui.view_widgets.get(&view_id) {
                    Some(fpas_std::ViewWidget::MenuBar(menu)) => Some(menu.query_state()),
                    Some(_) => None,
                    None => None,
                });
                let Some(state) = snapshot else {
                    return Err(query_menu_bar_state_error(view_id, line));
                };
                self.push(Self::tui_menu_bar_state_record(state))?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    pub(in crate::vm::execute::io) fn pop_query_view_id(
        &mut self,
        line: SourceLocation,
    ) -> Result<ViewId, VmError> {
        let view_id = self.pop_tui_view_id(line)?;
        self.require_registered_tui_view(view_id, line)
    }

    pub(in crate::vm::execute::io) fn screen_row_to_u16(
        y: i64,
        line: SourceLocation,
    ) -> Result<u16, VmError> {
        if y <= 0 || y > i64::from(u16::MAX) {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!(
                    "Application.QueryScreenLine(App, Y) requires Y in 1..={}, got {y}.",
                    u16::MAX
                ),
                "Pass a one-based row index within the virtual screen height.",
                line,
            ));
        }
        Ok(y as u16)
    }

    pub(in crate::vm::execute::io) fn screen_column_to_u16(
        x: i64,
        line: SourceLocation,
    ) -> Result<u16, VmError> {
        if x <= 0 || x > i64::from(u16::MAX) {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!(
                    "Application.QueryScreenCell(App, X, Y) requires X in 1..={}, got {x}.",
                    u16::MAX
                ),
                "Pass one-based column coordinates within the virtual screen width.",
                line,
            ));
        }
        Ok(x as u16)
    }
}

fn query_cell_error(x: u16, y: u16, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "Application.QueryScreenCell(App, {x}, {y}) is out of range or uses non-CRT colors."
        ),
        "Query cells inside the virtual screen after paint; v1 supports packed CRT colors only (0..=15).",
        line,
    )
}

fn query_view_rect_error(view_id: ViewId, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "Application.QueryViewRect(App, {}) could not resolve the view rectangle.",
            view_id.raw()
        ),
        "Pass a view handle returned by `Application.HostRegisterView` or a host widget constructor.",
        line,
    )
}

fn query_menu_bar_state_error(view_id: ViewId, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!(
            "Application.QueryMenuBarState(App, {}) requires a menu bar view handle.",
            view_id.raw()
        ),
        "Pass the view id returned by `Application.HostCreateMenuBarView`.",
        line,
    )
}
