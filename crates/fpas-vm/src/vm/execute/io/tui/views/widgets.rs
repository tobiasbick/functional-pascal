//! Native host-widget construction and model replacement intrinsics.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, TuiIntrinsic};
use fpas_std::{SolidFillWidget, ViewRect, ViewWidget, validate_packed_crt_color};

use super::super::view_geometry::validate_view_rect;

impl Worker {
    /// Executes native widget construction and model-replacement intrinsics.
    pub(super) fn try_exec_tui_view_widget_intrinsic(
        &mut self,
        intrinsic: TuiIntrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            TuiIntrinsic::HostCreateSolidFillView => {
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
                let view_rect = validate_view_rect(
                    "Application.HostCreateSolidFillView",
                    ViewRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    line,
                )?;
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
                self.push(Self::tui_view_id_record(view_id))?;
            }
            TuiIntrinsic::HostCreateStatusBarView => {
                let style = self.pop_status_bar_style(line)?;
                let segments = self.pop_status_bar_segments(line)?;
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let view_rect = validate_view_rect(
                    "Application.HostCreateStatusBarView",
                    ViewRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    line,
                )?;
                let widget = ViewWidget::StatusBar(fpas_std::StatusBarWidget::new(segments, style));
                let view_id = self.with_tui(|tui| {
                    let view_id = tui.views.register(view_rect);
                    tui.view_widgets.insert(view_id, widget);
                    let _ = tui.session.request_redraw_rect(view_rect, line);
                    view_id
                });
                self.push(Self::tui_view_id_record(view_id))?;
            }
            TuiIntrinsic::HostSetStatusBarSegments => {
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
}
