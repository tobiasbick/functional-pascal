//! Control construction, mutation, and query intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, TuiIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::{
    ButtonStyle, ButtonWidget, CheckBoxStyle, CheckBoxWidget, CommandId, InputLineStyle,
    InputLineWidget, LabelStyle, LabelWidget, RadioGroupStyle, RadioGroupWidget, ViewOptions,
    ViewRect, ViewWidget,
};

use super::super::view_geometry::validate_view_rect;

impl Worker {
    pub(in crate::vm::execute::io::tui) fn try_exec_tui_control_intrinsic(
        &mut self,
        intrinsic: TuiIntrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            TuiIntrinsic::HostCreateLabelView => {
                let accelerator = self.pop_optional_char("Accelerator", line)?;
                let text = self.pop_control_string("Text", line)?;
                let rect = self.pop_control_rect("HostCreateLabelView", line)?;
                self.create_control_view(
                    rect,
                    ViewWidget::Label(LabelWidget::new(text, accelerator, LabelStyle::default())),
                    false,
                    line,
                )?;
            }
            TuiIntrinsic::HostCreateButtonView => {
                let is_default = self.pop_bool(line)?;
                let command = self.pop_optional_integer("CommandId", line)?.map(CommandId);
                let caption = self.pop_control_string("Caption", line)?;
                let rect = self.pop_control_rect("HostCreateButtonView", line)?;
                let mut widget = ButtonWidget::new(caption, command, ButtonStyle::default());
                widget.default = is_default;
                self.create_control_view(rect, ViewWidget::Button(widget), true, line)?;
            }
            TuiIntrinsic::HostCreateInputLineView => {
                let text = self.pop_control_string("Text", line)?;
                let rect = self.pop_control_rect("HostCreateInputLineView", line)?;
                self.create_control_view(
                    rect,
                    ViewWidget::InputLine(InputLineWidget::new(text, InputLineStyle::default())),
                    true,
                    line,
                )?;
            }
            TuiIntrinsic::HostCreateCheckBoxView => {
                let checked = self.pop_bool(line)?;
                let command = self.pop_optional_integer("CommandId", line)?.map(CommandId);
                let accelerator = self.pop_optional_char("Accelerator", line)?;
                let label = self.pop_control_string("Label", line)?;
                let rect = self.pop_control_rect("HostCreateCheckBoxView", line)?;
                let mut widget =
                    CheckBoxWidget::new(label, accelerator, command, CheckBoxStyle::default());
                widget.checked = checked;
                self.create_control_view(rect, ViewWidget::CheckBox(widget), true, line)?;
            }
            TuiIntrinsic::HostCreateRadioGroupView => {
                let options = self.pop_radio_options(line)?;
                let rect = self.pop_control_rect("HostCreateRadioGroupView", line)?;
                self.create_control_view(
                    rect,
                    ViewWidget::RadioGroup(RadioGroupWidget::new(
                        options,
                        RadioGroupStyle::default(),
                    )),
                    true,
                    line,
                )?;
            }
            TuiIntrinsic::HostSetInputLineText => {
                let text = self.pop_control_string("Text", line)?;
                let id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.update_control(id, line, |w| {
                    if let ViewWidget::InputLine(v) = w {
                        v.set_text(text);
                        true
                    } else {
                        false
                    }
                });
            }
            TuiIntrinsic::HostSetCheckBoxChecked => {
                let checked = self.pop_bool(line)?;
                let id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.update_control(id, line, |w| {
                    if let ViewWidget::CheckBox(v) = w {
                        v.checked = checked;
                        true
                    } else {
                        false
                    }
                });
            }
            TuiIntrinsic::HostSetRadioGroupSelected => {
                let index = self.pop_int(line)?;
                let id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.update_control(id, line, |w| {
                    if let ViewWidget::RadioGroup(v) = w {
                        usize::try_from(index).is_ok_and(|i| v.set_selected(i))
                    } else {
                        false
                    }
                });
            }
            TuiIntrinsic::QueryInputLineState
            | TuiIntrinsic::QueryCheckBoxState
            | TuiIntrinsic::QueryRadioGroupState => {
                let id = self.pop_query_view_id(line)?;
                self.pop_tui_application(line)?;
                let value = self.with_tui(|tui| match (intrinsic, tui.view_widgets.get(&id)) {
                    (TuiIntrinsic::QueryInputLineState, Some(ViewWidget::InputLine(v))) => {
                        Some(control_record(
                            "Std.Tui.InputLineState",
                            vec![
                                ("text", Value::Str(v.text().into())),
                                ("cursor", Value::Integer(v.cursor() as i64)),
                                ("scrollOffset", Value::Integer(v.scroll_offset() as i64)),
                            ],
                        ))
                    }
                    (TuiIntrinsic::QueryCheckBoxState, Some(ViewWidget::CheckBox(v))) => {
                        Some(control_record(
                            "Std.Tui.CheckBoxState",
                            vec![("checked", Value::Boolean(v.checked))],
                        ))
                    }
                    (TuiIntrinsic::QueryRadioGroupState, Some(ViewWidget::RadioGroup(v))) => {
                        Some(control_record(
                            "Std.Tui.RadioGroupState",
                            vec![
                                (
                                    "selectedIndex",
                                    Value::Integer(v.selected().map_or(-1, |i| i as i64)),
                                ),
                                (
                                    "focusedIndex",
                                    Value::Integer(v.focused_option().map_or(-1, |i| i as i64)),
                                ),
                            ],
                        ))
                    }
                    _ => None,
                });
                let Some(value) = value else {
                    return Err(runtime_error(
                        RUNTIME_CONSOLE_STATE_ERROR,
                        "Control query received the wrong view kind",
                        "Pass the matching control ViewId.",
                        line,
                    ));
                };
                self.push(value)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn pop_control_rect(
        &mut self,
        operation: &str,
        line: SourceLocation,
    ) -> Result<ViewRect, VmError> {
        let height = self.pop_int(line)?;
        let width = self.pop_int(line)?;
        let y = self.pop_int(line)?;
        let x = self.pop_int(line)?;
        self.pop_tui_application(line)?;
        validate_view_rect(
            &format!("Application.{operation}"),
            ViewRect {
                x,
                y,
                width,
                height,
            },
            line,
        )
    }
    fn create_control_view(
        &mut self,
        rect: ViewRect,
        widget: ViewWidget,
        focusable: bool,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let id = self.with_tui(|tui| {
            let options = ViewOptions {
                selectable: focusable,
                tab_stop: focusable,
                ..ViewOptions::default()
            };
            let id = tui.views.register_with_options(rect, options);
            tui.view_widgets.insert(id, widget);
            let _ = tui.session.request_redraw_rect(rect, line);
            id
        });
        self.push(Self::tui_view_id_record(id))
    }
    fn update_control(
        &mut self,
        id: fpas_std::ViewId,
        line: SourceLocation,
        update: impl FnOnce(&mut ViewWidget) -> bool,
    ) {
        self.with_tui(|tui| {
            if let Some(widget) = tui.view_widgets.get_mut(&id)
                && update(widget)
                && let Some(rect) = tui.views.rect(id)
            {
                let _ = tui.session.request_redraw_rect(rect, line);
            }
        });
    }
}

fn control_record(type_name: &str, fields: Vec<(&str, Value)>) -> Value {
    Value::Record {
        type_name: type_name.into(),
        fields: fields.into_iter().map(|(n, v)| (n.into(), v)).collect(),
    }
}
