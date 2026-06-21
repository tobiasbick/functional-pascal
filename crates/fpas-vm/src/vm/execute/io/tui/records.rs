//! `Std.Tui` value constructors: `Application`, `ViewId`, `Size`, and `Rect` records.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::{MenuBarState, ResolvedView, ViewId, ViewKind, ViewOptions, ViewRect, ViewState};

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
const TUI_VIEW_ID_TYPE: &str = "Std.Tui.ViewId";
const TUI_VIEW_ID_RAW_FIELD: &str = "__id";
const TUI_RECT_TYPE: &str = "Std.Tui.Rect";
const TUI_SIZE_TYPE: &str = "Std.Tui.Size";
const TUI_SCREEN_CELL_TYPE: &str = "Std.Tui.ScreenCell";
const TUI_MENU_BAR_STATE_TYPE: &str = "Std.Tui.MenuBarState";
const TUI_VIEW_STATE_TYPE: &str = "Std.Tui.ViewState";
const TUI_VIEW_OPTIONS_TYPE: &str = "Std.Tui.ViewOptions";
const TUI_RESOLVED_VIEW_TYPE: &str = "Std.Tui.ResolvedView";
const TUI_VIEW_SNAPSHOT_TYPE: &str = "Std.Tui.ViewSnapshot";

impl Worker {
    /// Constructs an empty `Std.Tui.Application` record.
    pub(in crate::vm::execute::io) fn tui_application_record() -> Value {
        Value::Record {
            type_name: TUI_APPLICATION_TYPE.into(),
            fields: vec![],
        }
    }

    /// Constructs a `Std.Tui.ViewId` record backed by the host registry token.
    pub(in crate::vm::execute::io) fn tui_view_id_record(view_id: ViewId) -> Value {
        Value::Record {
            type_name: TUI_VIEW_ID_TYPE.into(),
            fields: vec![(
                TUI_VIEW_ID_RAW_FIELD.into(),
                Value::Integer(i64::from(view_id.raw())),
            )],
        }
    }

    /// Reads the host token from a `Std.Tui.ViewId` runtime value.
    pub(in crate::vm::execute::io) fn tui_view_id_from_value(
        value: &Value,
        line: SourceLocation,
    ) -> Result<ViewId, VmError> {
        match value {
            Value::Record { type_name, fields } if type_name == TUI_VIEW_ID_TYPE => {
                let Some(Value::Integer(raw)) = fields
                    .iter()
                    .find(|(name, _)| name == TUI_VIEW_ID_RAW_FIELD)
                    .map(|(_, value)| value)
                else {
                    return Err(runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "Std.Tui.ViewId is missing its internal host token",
                        "Pass a view handle returned by `Application.HostRegisterView` or a host widget constructor.",
                        line,
                    ));
                };
                if *raw < 0 {
                    return Err(runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        format!("ViewId host token {raw} is out of range"),
                        "Pass a view handle returned by `Application.HostRegisterView` or a host widget constructor.",
                        line,
                    ));
                }
                Ok(ViewId::from_raw(*raw as u32))
            }
            other => Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Expected Std.Tui.ViewId, got {}", other.type_name()),
                "Pass a view handle returned by `Application.HostRegisterView` or a host widget constructor.",
                line,
            )),
        }
    }

    /// Constructs a `Std.Tui.Size` record with `width` and `height` fields.
    pub(in crate::vm::execute::io) fn tui_size_record(width: i64, height: i64) -> Value {
        Value::Record {
            type_name: TUI_SIZE_TYPE.into(),
            fields: vec![
                ("width".into(), Value::Integer(width)),
                ("height".into(), Value::Integer(height)),
            ],
        }
    }

    /// Constructs a `Std.Tui.ScreenCell` record with `ch`, `fg`, and `bg` fields.
    pub(in crate::vm::execute::io) fn tui_screen_cell_record(ch: char, fg: u8, bg: u8) -> Value {
        Value::Record {
            type_name: TUI_SCREEN_CELL_TYPE.into(),
            fields: vec![
                ("ch".into(), Value::Str(ch.to_string())),
                ("fg".into(), Value::Integer(i64::from(fg))),
                ("bg".into(), Value::Integer(i64::from(bg))),
            ],
        }
    }

    /// Constructs a `Std.Tui.Rect` record with `x`, `y`, `width`, and `height` fields.
    pub(in crate::vm::execute::io) fn tui_rect_record(rect: ViewRect) -> Value {
        Value::Record {
            type_name: TUI_RECT_TYPE.into(),
            fields: vec![
                ("x".into(), Value::Integer(rect.x)),
                ("y".into(), Value::Integer(rect.y)),
                ("width".into(), Value::Integer(rect.width)),
                ("height".into(), Value::Integer(rect.height)),
            ],
        }
    }

    /// Constructs a `Std.Tui.MenuBarState` record from a widget snapshot.
    pub(in crate::vm::execute::io) fn tui_menu_bar_state_record(state: MenuBarState) -> Value {
        Value::Record {
            type_name: TUI_MENU_BAR_STATE_TYPE.into(),
            fields: vec![
                ("menuActive".into(), Value::Boolean(state.menu_active)),
                ("hoveredIndex".into(), Value::Integer(state.hovered_index)),
                ("submenuOpen".into(), Value::Boolean(state.submenu_open)),
                (
                    "submenuBarIndex".into(),
                    Value::Integer(state.submenu_bar_index),
                ),
                ("selectedEntry".into(), Value::Integer(state.selected_entry)),
            ],
        }
    }

    /// Constructs a `Std.Tui.ViewState` record from resolved retained state.
    pub(in crate::vm::execute::io) fn tui_view_state_record(state: ViewState) -> Value {
        Value::Record {
            type_name: TUI_VIEW_STATE_TYPE.into(),
            fields: vec![
                ("visible".into(), Value::Boolean(state.visible)),
                ("enabled".into(), Value::Boolean(state.enabled)),
                ("focused".into(), Value::Boolean(state.focused)),
                ("active".into(), Value::Boolean(state.active)),
                ("exposed".into(), Value::Boolean(state.exposed)),
            ],
        }
    }

    /// Constructs a `Std.Tui.ViewOptions` record from retained behavior options.
    pub(in crate::vm::execute::io) fn tui_view_options_record(options: ViewOptions) -> Value {
        Value::Record {
            type_name: TUI_VIEW_OPTIONS_TYPE.into(),
            fields: vec![
                ("selectable".into(), Value::Boolean(options.selectable)),
                ("tabStop".into(), Value::Boolean(options.tab_stop)),
                ("preProcess".into(), Value::Boolean(options.pre_process)),
                ("postProcess".into(), Value::Boolean(options.post_process)),
                ("clipChildren".into(), Value::Boolean(options.clip_children)),
            ],
        }
    }

    /// Constructs a `Std.Tui.ResolvedView` record.
    pub(in crate::vm::execute::io) fn tui_resolved_view_record(view: ResolvedView) -> Value {
        Value::Record {
            type_name: TUI_RESOLVED_VIEW_TYPE.into(),
            fields: vec![
                ("rect".into(), Self::tui_rect_record(view.rect)),
                (
                    "clip".into(),
                    view.clip.map_or(Value::OptionNone, |clip| {
                        Value::OptionSome(Box::new(Self::tui_rect_record(clip)))
                    }),
                ),
                ("state".into(), Self::tui_view_state_record(view.state)),
                (
                    "options".into(),
                    Self::tui_view_options_record(view.options),
                ),
            ],
        }
    }

    /// Constructs the runtime enum discriminant for `Std.Tui.ViewKind`.
    pub(in crate::vm::execute::io) fn tui_view_kind_value(kind: ViewKind) -> Value {
        Value::Integer(kind as i64)
    }

    /// Constructs one `Std.Tui.ViewSnapshot` scene-graph entry.
    pub(in crate::vm::execute::io) fn tui_view_snapshot_record(
        view: ResolvedView,
        parent: Option<ViewId>,
        children: &[ViewId],
        kind: ViewKind,
    ) -> Value {
        Value::Record {
            type_name: TUI_VIEW_SNAPSHOT_TYPE.into(),
            fields: vec![
                ("id".into(), Self::tui_view_id_record(view.id)),
                (
                    "parent".into(),
                    parent.map_or(Value::OptionNone, |id| {
                        Value::OptionSome(Box::new(Self::tui_view_id_record(id)))
                    }),
                ),
                (
                    "children".into(),
                    Value::Array(
                        children
                            .iter()
                            .map(|id| Self::tui_view_id_record(*id))
                            .collect(),
                    ),
                ),
                ("resolved".into(), Self::tui_resolved_view_record(view)),
                ("kind".into(), Self::tui_view_kind_value(kind)),
            ],
        }
    }
}
