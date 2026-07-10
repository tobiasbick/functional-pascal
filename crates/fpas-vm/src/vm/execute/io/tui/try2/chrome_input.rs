//! Turbo Vision menu bar and status line bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, runtime_error};
use crate::vm::shared::{TurboVisionMenu, TurboVisionMenuItem, TurboVisionStatusItem};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

const TUI_MENU_TYPE: &str = "Std.Tui.Menu";
const TUI_MENU_ITEM_TYPE: &str = "Std.Tui.MenuItem";
const TUI_STATUS_ITEM_TYPE: &str = "Std.Tui.StatusItem";

impl Worker {
    pub(in crate::vm::execute::io::tui) fn turbo_vision_set_menu_bar(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let handle = self.pop_turbo_vision_menu_bar_handle(line)?;
        self.pop_tui_application(line)?;
        super::try2_set_menu_bar(self, handle, line)
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_set_status_line(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let handle = self.pop_turbo_vision_status_line_handle(line)?;
        self.pop_tui_application(line)?;
        super::try2_set_status_line(self, handle, line)
    }

    /// Parses menu records for try-2 `MenuBar.New`.
    pub(in crate::vm::execute::io::tui) fn parse_turbo_vision_menus(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<TurboVisionMenu>, VmError> {
        self.pop_turbo_vision_menus(line)
    }

    /// Parses status items for try-2 `StatusLine.New`.
    pub(in crate::vm::execute::io::tui) fn parse_turbo_vision_status_items(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<TurboVisionStatusItem>, VmError> {
        self.pop_turbo_vision_status_items(line)
    }

    fn pop_turbo_vision_menus(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<TurboVisionMenu>, VmError> {
        let value = self.pop(line)?;
        let Value::Array(values) = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("MenuBar Menus must be array, got {}", value.type_name()),
                "Pass an array of Std.Tui.Menu records.",
                line,
            ));
        };

        values
            .into_iter()
            .map(|value| {
                let fields = expect_record(value, TUI_MENU_TYPE, "Menu", line)?;
                let title = string_field(&fields, "title", "Menu", line)?;
                let items = menu_items_field(&fields, line)?;
                Ok(TurboVisionMenu { title, items })
            })
            .collect()
    }

    fn pop_turbo_vision_status_items(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<TurboVisionStatusItem>, VmError> {
        let value = self.pop(line)?;
        let Value::Array(values) = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("StatusLine Items must be array, got {}", value.type_name()),
                "Pass an array of Std.Tui.StatusItem records.",
                line,
            ));
        };

        values
            .into_iter()
            .map(|value| {
                let fields = expect_record(value, TUI_STATUS_ITEM_TYPE, "StatusItem", line)?;
                let text = string_field(&fields, "text", "StatusItem", line)?;
                let key_code = u16_field(&fields, "keyCode", "StatusItem key code", line)?;
                let command_id = u16_field(&fields, "commandId", "StatusItem command id", line)?;
                Ok(TurboVisionStatusItem {
                    text,
                    key_code,
                    command_id,
                })
            })
            .collect()
    }
}

fn menu_items_field(
    fields: &[(String, Value)],
    line: SourceLocation,
) -> Result<Vec<TurboVisionMenuItem>, VmError> {
    let value = fields
        .iter()
        .find(|(name, _)| name == "items")
        .map(|(_, value)| value)
        .ok_or_else(|| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Menu.items is missing",
                "Pass a `Menu` record with an `items` array of `MenuItem` records.",
                line,
            )
        })?;
    let Value::Array(values) = value else {
        return Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!("Menu.items must be array, got {}", value.type_name()),
            "Pass an array of Std.Tui.MenuItem records.",
            line,
        ));
    };

    values
        .iter()
        .map(|value| {
            let fields = expect_record(value.clone(), TUI_MENU_ITEM_TYPE, "MenuItem", line)?;
            let text = string_field(&fields, "text", "MenuItem", line)?;
            let command_id = u16_field(&fields, "commandId", "MenuItem command id", line)?;
            Ok(TurboVisionMenuItem { text, command_id })
        })
        .collect()
}

fn expect_record(
    value: Value,
    expected_type: &'static str,
    label: &'static str,
    line: SourceLocation,
) -> Result<Vec<(String, Value)>, VmError> {
    let Value::Record { type_name, fields } = value else {
        return Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!(
                "{label} expected {expected_type}, got {}",
                value.type_name()
            ),
            format!("Pass a {expected_type} record."),
            line,
        ));
    };
    if type_name != expected_type {
        return Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!("{label} expected {expected_type}, got {type_name}"),
            format!("Pass a {expected_type} record."),
            line,
        ));
    }
    Ok(fields)
}

fn string_field(
    fields: &[(String, Value)],
    field_name: &'static str,
    label: &'static str,
    line: SourceLocation,
) -> Result<String, VmError> {
    match fields.iter().find(|(name, _)| name == field_name) {
        Some((_, Value::Str(value))) => Ok(value.clone()),
        Some((_, other)) => Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!(
                "{label}.{field_name} must be string, got {}",
                other.type_name()
            ),
            "Use a string field value.",
            line,
        )),
        None => Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("{label}.{field_name} is missing"),
            "Use the Std.Tui record field names documented for this type.",
            line,
        )),
    }
}

fn u16_field(
    fields: &[(String, Value)],
    field_name: &'static str,
    label: &'static str,
    line: SourceLocation,
) -> Result<u16, VmError> {
    let raw = match fields.iter().find(|(name, _)| name == field_name) {
        Some((_, Value::Integer(value))) => *value,
        Some((_, other)) => {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("{label} must be integer, got {}", other.type_name()),
                "Use an integer in the range 0..65535.",
                line,
            ));
        }
        None => {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("{label} is missing"),
                "Use the Std.Tui record field names documented for this type.",
                line,
            ));
        }
    };
    u16::try_from(raw).map_err(|_| {
        runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("{label} is outside the Turbo Vision u16 range"),
            "Use an integer in the range 0..65535.",
            line,
        )
    })
}
