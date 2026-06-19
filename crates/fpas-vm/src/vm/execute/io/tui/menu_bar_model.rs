//! Decode Pascal menu bar models from VM values.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::runtime_error;
use crate::vm::shared::TuiState;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{
    MenuBarItem, MenuBarMouseResult, MenuBarStyle, MenuPopupItem, ViewId, ViewRect, ViewWidget,
    validate_packed_crt_color,
};

use super::widget_target;

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
            Value::Char(ch) if *ch == '\0' => String::new(),
            Value::Char(ch) => ch.to_string(),
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "MenuBarItem.Shortcut must be char, got {}",
                        other.type_name()
                    ),
                    "Set `Shortcut := 'F'` for Alt+F, or `Shortcut := #0` when none.",
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
            Value::Char(ch) if *ch == '\0' => String::new(),
            Value::Char(ch) => ch.to_string(),
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "MenuPopupItem.Shortcut must be char, got {}",
                        other.type_name()
                    ),
                    "Set `Shortcut := 'X'` for a popup shortcut, or `Shortcut := #0` when none.",
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

    /// Routes a mouse event to host widgets before Pascal `OnMouse` handlers run.
    pub(in crate::vm::execute::io::tui) fn try_dispatch_widget_mouse(
        &mut self,
        mouse: fpas_std::UiMouse,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<Option<i64>, VmError> {
        let hit = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            widget_target::widget_mouse_hit(&tui.views, &tui.view_widgets, mouse, modal_scope)
        };

        let Some((view_id, rect, mut widget)) = hit else {
            return Ok(None);
        };

        let before = match &widget {
            ViewWidget::MenuBar(menu) => menu.damage_rects(rect),
            _ => vec![rect],
        };

        let result = match &mut widget {
            ViewWidget::MenuBar(menu) => menu.handle_mouse(rect, mouse),
            ViewWidget::SolidFill(_) | ViewWidget::StatusBar(_) => MenuBarMouseResult::Ignored,
        };

        let after = match &widget {
            ViewWidget::MenuBar(menu) => menu.damage_rects(rect),
            _ => vec![rect],
        };

        let dispatch_tag = match result {
            MenuBarMouseResult::Ignored => return Ok(None),
            MenuBarMouseResult::HoverChanged => {
                self.with_tui(|tui| {
                    tui.view_widgets.insert(view_id, widget);
                    let mut regions = before;
                    regions.extend(after);
                    Self::request_unique_redraws(tui, &regions, line);
                });
                5
            }
            MenuBarMouseResult::Command(command_id) => {
                self.with_tui(|tui| {
                    tui.view_widgets.insert(view_id, widget);
                    let mut regions = before;
                    regions.extend(after);
                    Self::request_unique_redraws(tui, &regions, line);
                });
                return self.dispatch_tui_command(command_id, line).map(Some);
            }
        };

        Ok(Some(dispatch_tag))
    }

    /// Routes keyboard shortcuts to host menu bar widgets before global command bindings.
    pub(in crate::vm::execute::io::tui) fn try_dispatch_widget_key(
        &mut self,
        key: fpas_std::ConsoleKeyEvent,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<Option<i64>, VmError> {
        let hit = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            widget_target::topmost_menu_bar(&tui.views, &tui.view_widgets, modal_scope)
        };

        let Some((view_id, rect, mut widget)) = hit else {
            return Ok(None);
        };

        let ViewWidget::MenuBar(menu) = &widget else {
            return Ok(None);
        };
        let before = menu.damage_rects(rect);

        let result = match &mut widget {
            ViewWidget::MenuBar(menu) => menu.handle_key(&key),
            _ => unreachable!(),
        };
        let after = match &widget {
            ViewWidget::MenuBar(menu) => menu.damage_rects(rect),
            _ => vec![rect],
        };
        let dispatch_tag = match result {
            MenuBarMouseResult::Ignored => return Ok(None),
            MenuBarMouseResult::HoverChanged => {
                self.with_tui(|tui| {
                    tui.view_widgets.insert(view_id, widget);
                    let mut regions = before;
                    regions.extend(after);
                    Self::request_unique_redraws(tui, &regions, line);
                });
                21
            }
            MenuBarMouseResult::Command(command_id) => {
                self.with_tui(|tui| {
                    tui.view_widgets.insert(view_id, widget);
                    let mut regions = before;
                    regions.extend(after);
                    Self::request_unique_redraws(tui, &regions, line);
                });
                return self.dispatch_tui_command(command_id, line).map(Some);
            }
        };

        Ok(Some(dispatch_tag))
    }

    fn request_unique_redraws(tui: &mut TuiState, regions: &[ViewRect], line: SourceLocation) {
        let mut unique = Vec::new();
        for region in regions {
            if unique.iter().any(|existing: &ViewRect| existing == region) {
                continue;
            }
            unique.push(*region);
        }
        for region in unique {
            let _ = tui.session.request_redraw_rect(region, line);
        }
    }
}
