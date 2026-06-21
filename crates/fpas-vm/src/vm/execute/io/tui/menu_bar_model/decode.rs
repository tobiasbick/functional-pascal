//! Decode Pascal menu-bar records from VM values.

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::runtime_error;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{MenuBarItem, MenuBarStyle, MenuPopupItem, validate_packed_crt_color};

const MENU_BAR_ITEM_TYPE: &str = "Std.Tui.MenuBarItem";
const MENU_BAR_STYLE_TYPE: &str = "Std.Tui.MenuBarStyle";
const MENU_POPUP_ITEM_TYPE: &str = "Std.Tui.MenuPopupItem";

impl Worker {
    /// Parses `array of MenuBarItem` from the stack top.
    pub(in crate::vm::execute::io::tui) fn pop_menu_bar_items(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<MenuBarItem>, VmError> {
        match self.pop(line)? {
            Value::Array(values) => values
                .into_iter()
                .map(|value| self.decode_menu_bar_item(&value, line))
                .collect(),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected array of MenuBarItem, got {}", other.type_name()),
                "Pass an array literal such as `[record Label := 'File'; ... end]`.",
                line,
            )),
        }
    }

    /// Parses `MenuBarStyle` from the stack top.
    pub(in crate::vm::execute::io::tui) fn pop_menu_bar_style(
        &mut self,
        line: SourceLocation,
    ) -> Result<MenuBarStyle, VmError> {
        let value = self.pop(line)?;
        self.decode_menu_bar_style(&value, line)
    }

    fn decode_menu_bar_item(
        &self,
        value: &Value,
        line: SourceLocation,
    ) -> Result<MenuBarItem, VmError> {
        let Value::Record { type_name, fields } = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {MENU_BAR_ITEM_TYPE}, got {}", value.type_name()),
                "Each menu entry must be a `MenuBarItem` record.",
                line,
            ));
        };
        if type_name != MENU_BAR_ITEM_TYPE {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {MENU_BAR_ITEM_TYPE}, got `{type_name}`"),
                "Each menu entry must be a `MenuBarItem` record.",
                line,
            ));
        }

        let label = match Self::required_record_field(fields, "Label", line)? {
            Value::Str(label) => label.clone(),
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "MenuBarItem.Label must be string, got {}",
                        other.type_name()
                    ),
                    "Set `Label := 'File'` with a string literal.",
                    line,
                ));
            }
        };
        let shortcut = match Self::required_record_field(fields, "Shortcut", line)? {
            Value::Str(shortcut) => shortcut.clone(),
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "MenuBarItem.Shortcut must be string, got {}",
                        other.type_name()
                    ),
                    "Set `Shortcut := 'F'` for Alt+F, or `Shortcut := ''` when none.",
                    line,
                ));
            }
        };
        let enabled = match Self::required_record_field(fields, "Enabled", line)? {
            Value::Boolean(flag) => *flag,
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "MenuBarItem.Enabled must be boolean, got {}",
                        other.type_name()
                    ),
                    "Set `Enabled := true` or `Enabled := false`.",
                    line,
                ));
            }
        };
        let command_id = self.integer_record_field(fields, "CommandId", line)?;
        let submenu = self.decode_menu_popup_items(fields, "Submenu", line)?;

        Ok(MenuBarItem {
            label,
            shortcut,
            enabled,
            command_id,
            submenu,
        })
    }

    fn decode_menu_popup_items(
        &self,
        fields: &[(String, Value)],
        field_name: &str,
        line: SourceLocation,
    ) -> Result<Vec<MenuPopupItem>, VmError> {
        match fields.iter().find(|(name, _)| name == field_name) {
            None => Ok(Vec::new()),
            Some((_, Value::Array(values))) => values
                .iter()
                .map(|value| self.decode_menu_popup_item(value, line))
                .collect(),
            Some((_, other)) => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "MenuBarItem.{field_name} must be array of MenuPopupItem, got {}",
                    other.type_name()
                ),
                "Set `Submenu := [record Label := 'Exit'; ... end]` or `Submenu := []`.",
                line,
            )),
        }
    }

    fn decode_menu_popup_item(
        &self,
        value: &Value,
        line: SourceLocation,
    ) -> Result<MenuPopupItem, VmError> {
        let Value::Record { type_name, fields } = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {MENU_POPUP_ITEM_TYPE}, got {}", value.type_name()),
                "Each submenu entry must be a `MenuPopupItem` record.",
                line,
            ));
        };
        if type_name != MENU_POPUP_ITEM_TYPE {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {MENU_POPUP_ITEM_TYPE}, got `{type_name}`"),
                "Each submenu entry must be a `MenuPopupItem` record.",
                line,
            ));
        }

        let label = match Self::required_record_field(fields, "Label", line)? {
            Value::Str(label) => label.clone(),
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "MenuPopupItem.Label must be string, got {}",
                        other.type_name()
                    ),
                    "Set `Label := 'Exit'` with a string literal.",
                    line,
                ));
            }
        };
        let shortcut = match Self::required_record_field(fields, "Shortcut", line)? {
            Value::Str(shortcut) => shortcut.clone(),
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "MenuPopupItem.Shortcut must be string, got {}",
                        other.type_name()
                    ),
                    "Set `Shortcut := 'X'` for a popup shortcut, or `Shortcut := ''` when none.",
                    line,
                ));
            }
        };
        let enabled = match Self::required_record_field(fields, "Enabled", line)? {
            Value::Boolean(flag) => *flag,
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "MenuPopupItem.Enabled must be boolean, got {}",
                        other.type_name()
                    ),
                    "Set `Enabled := true` or `Enabled := false`.",
                    line,
                ));
            }
        };
        let command_id = self.integer_record_field(fields, "CommandId", line)?;
        let separator = match fields.iter().find(|(name, _)| name == "Separator") {
            Some((_, Value::Boolean(flag))) => *flag,
            _ => false,
        };

        Ok(MenuPopupItem {
            label,
            shortcut,
            enabled,
            command_id,
            separator,
        })
    }

    fn decode_menu_bar_style(
        &self,
        value: &Value,
        line: SourceLocation,
    ) -> Result<MenuBarStyle, VmError> {
        let Value::Record { type_name, fields } = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {MENU_BAR_STYLE_TYPE}, got {}", value.type_name()),
                "Pass a `MenuBarStyle` record with CRT color indices.",
                line,
            ));
        };
        if type_name != MENU_BAR_STYLE_TYPE {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {MENU_BAR_STYLE_TYPE}, got `{type_name}`"),
                "Pass a `MenuBarStyle` record with CRT color indices.",
                line,
            ));
        }

        let bar_bg = validate_packed_crt_color(
            self.integer_record_field(fields, "BarBg", line)?,
            "MenuBarStyle.BarBg",
            line,
        )?;
        let bar_fg = validate_packed_crt_color(
            self.integer_record_field(fields, "BarFg", line)?,
            "MenuBarStyle.BarFg",
            line,
        )?;
        let accel_fg = validate_packed_crt_color(
            self.integer_record_field(fields, "AccelFg", line)?,
            "MenuBarStyle.AccelFg",
            line,
        )?;
        let highlight_bg = validate_packed_crt_color(
            self.integer_record_field(fields, "HighlightBg", line)?,
            "MenuBarStyle.HighlightBg",
            line,
        )?;
        let highlight_fg = validate_packed_crt_color(
            self.integer_record_field(fields, "HighlightFg", line)?,
            "MenuBarStyle.HighlightFg",
            line,
        )?;
        let disabled_fg = validate_packed_crt_color(
            self.integer_record_field(fields, "DisabledFg", line)?,
            "MenuBarStyle.DisabledFg",
            line,
        )?;

        Ok(MenuBarStyle {
            bar_bg,
            bar_fg,
            accel_fg,
            highlight_bg,
            highlight_fg,
            disabled_fg,
        })
    }
}
