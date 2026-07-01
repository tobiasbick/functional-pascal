//! Turbo Vision menu bar and status line bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::menu_build::build_menu_bar;
use super::tv_geometry::unknown_handle_error;
use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, runtime_error};
use crate::vm::shared::{
    TurboVisionMenu, TurboVisionMenuBar, TurboVisionMenuItem, TurboVisionObject,
    TurboVisionStatusItem, TurboVisionStatusLine,
};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::views::status_line::{StatusItem, StatusLine};

const TUI_MENU_TYPE: &str = "Std.Tui.Menu";
const TUI_MENU_ITEM_TYPE: &str = "Std.Tui.MenuItem";
const TUI_STATUS_ITEM_TYPE: &str = "Std.Tui.StatusItem";

impl Worker {
    pub(super) fn turbo_vision_create_menu_bar(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let menus = self.pop_turbo_vision_menus(line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let _menu_bar = build_menu_bar(bounds, &menus);

        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::MenuBar(TurboVisionMenuBar {
                    bounds,
                    menus,
                    attached: false,
                }),
            );
            handle
        });
        self.push(Self::turbo_vision_menu_bar_record(handle))
    }

    pub(super) fn turbo_vision_set_menu_bar(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let handle = self.pop_turbo_vision_menu_bar_handle(line)?;
        self.pop_tui_application(line)?;

        self.with_tui(|tui| {
            let Some(TurboVisionObject::MenuBar(menu_bar)) =
                tui.turbo_vision.objects.get_mut(&handle)
            else {
                return Err(unknown_handle_error("MenuBar", handle, line));
            };
            if menu_bar.attached {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("MenuBar handle {handle} is already attached"),
                    "Only set a Turbo Vision menu bar as application chrome once.",
                    line,
                ));
            }
            if tui.turbo_vision.menu_bar.is_some() {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Application already has a Turbo Vision menu bar",
                    "Set only one menu bar for the active application session.",
                    line,
                ));
            }
            menu_bar.attached = true;
            tui.turbo_vision.menu_bar = Some(handle);
            Ok(())
        })
    }

    pub(super) fn turbo_vision_create_status_line(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let items = self.pop_turbo_vision_status_items(line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        let _status_line = StatusLine::new(
            bounds,
            items
                .iter()
                .map(|item| StatusItem::new(&item.text, item.key_code, item.command_id))
                .collect(),
        );

        let bounds = super::tv_geometry::state_rect(bounds);
        let handle = self.with_tui(|tui| {
            let handle = tui.turbo_vision.next_handle;
            tui.turbo_vision.next_handle = handle.saturating_add(1).max(1);
            tui.turbo_vision.objects.insert(
                handle,
                TurboVisionObject::StatusLine(TurboVisionStatusLine {
                    bounds,
                    items,
                    attached: false,
                }),
            );
            handle
        });
        self.push(Self::turbo_vision_status_line_record(handle))
    }

    pub(super) fn turbo_vision_set_status_line(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let handle = self.pop_turbo_vision_status_line_handle(line)?;
        self.pop_tui_application(line)?;

        self.with_tui(|tui| {
            let Some(TurboVisionObject::StatusLine(status_line)) =
                tui.turbo_vision.objects.get_mut(&handle)
            else {
                return Err(unknown_handle_error("StatusLine", handle, line));
            };
            if status_line.attached {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("StatusLine handle {handle} is already attached"),
                    "Only set a Turbo Vision status line as application chrome once.",
                    line,
                ));
            }
            if tui.turbo_vision.status_line.is_some() {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Application already has a Turbo Vision status line",
                    "Set only one status line for the active application session.",
                    line,
                ));
            }
            status_line.attached = true;
            tui.turbo_vision.status_line = Some(handle);
            Ok(())
        })
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
