//! TOML sidecar parsing for test scripts.
//!
//! **Documentation:** [`docs/future/test-framework/scripted-input.md`](../../../docs/future/test-framework/scripted-input.md)

use std::fs;
use std::path::{Path, PathBuf};

/// Optional script-wide defaults from `[config]`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptConfig {
    /// When true, graph events are allowed (Phase 4 runner integration).
    pub headless_graph: bool,
}

/// One scripted input event before `vm.run()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptEvent {
    Readln {
        line: String,
    },
    ReadkeyChars {
        chars: String,
    },
    ConsoleKey {
        kind: String,
        ch: Option<char>,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
    ConsoleMouse {
        action: String,
        button: String,
        x: i64,
        y: i64,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
    ConsoleResize {
        width: i64,
        height: i64,
    },
    ConsolePaste {
        text: String,
    },
    ConsoleFocusGained,
    ConsoleFocusLost,
    GraphKey {
        kind: String,
        ch: Option<char>,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
    GraphMouse {
        action: String,
        button: String,
        x: i64,
        y: i64,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
    GraphWheel {
        delta_x: i64,
        delta_y: i64,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
}

/// Parsed sidecar script contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptFile {
    pub config: ScriptConfig,
    pub events: Vec<ScriptEvent>,
}

/// Returns the default sidecar path for a test source file (`*_test.fpas` → `*.script.toml`).
pub fn sidecar_path_for_test(test_path: &Path) -> PathBuf {
    test_path.with_extension("script.toml")
}

/// Reads and parses a script file from disk.
pub fn load_script(path: &Path) -> Result<ScriptFile, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "Error reading script `{}`: {error}\n  help: Sidecar scripts use `<test>.script.toml`.",
            path.display()
        )
    })?;
    parse_script_text(&text, path)
}

/// Parses script TOML from a string.
pub fn parse_script_text(text: &str, path: &Path) -> Result<ScriptFile, String> {
    let root: toml::Table = toml::from_str(text).map_err(|error| {
        format!(
            "Invalid script `{}`: {error}\n  help: See docs/future/test-framework/scripted-input.md.",
            path.display()
        )
    })?;

    let config = parse_config(root.get("config"), path)?;
    let events = parse_events(root.get("event"), path)?;
    Ok(ScriptFile { config, events })
}

fn parse_config(value: Option<&toml::Value>, path: &Path) -> Result<ScriptConfig, String> {
    let Some(value) = value else {
        return Ok(ScriptConfig::default());
    };
    let table = value.as_table().ok_or_else(|| {
        format!(
            "Invalid `[config]` in `{}`: expected a table.\n  help: Use `[config] headless_graph = false`.",
            path.display()
        )
    })?;
    let headless_graph = table
        .get("headless_graph")
        .map(parse_bool_field)
        .transpose()
        .map_err(|error| {
            format!(
                "Invalid `[config].headless_graph` in `{}`: {error}",
                path.display()
            )
        })?
        .unwrap_or(false);
    Ok(ScriptConfig { headless_graph })
}

fn parse_events(value: Option<&toml::Value>, path: &Path) -> Result<Vec<ScriptEvent>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let array = value.as_array().ok_or_else(|| {
        format!(
            "Invalid `[[event]]` in `{}`: expected an array of tables.\n  help: Each event needs `type = \"readln\"` (or another supported type).",
            path.display()
        )
    })?;

    let mut events = Vec::with_capacity(array.len());
    for (index, item) in array.iter().enumerate() {
        let table = item.as_table().ok_or_else(|| {
            format!(
                "Invalid `[[event]]` entry #{index} in `{}`: expected a table.",
                path.display()
            )
        })?;
        events.push(parse_event_table(table, index, path)?);
    }
    Ok(events)
}

fn parse_event_table(
    table: &toml::Table,
    index: usize,
    path: &Path,
) -> Result<ScriptEvent, String> {
    let event_type = required_string(table, "type", index, path)?;
    match event_type.as_str() {
        "readln" => Ok(ScriptEvent::Readln {
            line: required_string(table, "line", index, path)?,
        }),
        "readkey_chars" => Ok(ScriptEvent::ReadkeyChars {
            chars: required_string(table, "chars", index, path)?,
        }),
        "console_key" => Ok(ScriptEvent::ConsoleKey {
            kind: required_string(table, "kind", index, path)?,
            ch: optional_char(table, "ch", index, path)?,
            shift: optional_bool(table, "shift").unwrap_or(false),
            ctrl: optional_bool(table, "ctrl").unwrap_or(false),
            alt: optional_bool(table, "alt").unwrap_or(false),
            meta: optional_bool(table, "meta").unwrap_or(false),
        }),
        "console_mouse" => Ok(ScriptEvent::ConsoleMouse {
            action: required_string(table, "action", index, path)?,
            button: required_string(table, "button", index, path)?,
            x: required_i64(table, "x", index, path)?,
            y: required_i64(table, "y", index, path)?,
            shift: optional_bool(table, "shift").unwrap_or(false),
            ctrl: optional_bool(table, "ctrl").unwrap_or(false),
            alt: optional_bool(table, "alt").unwrap_or(false),
            meta: optional_bool(table, "meta").unwrap_or(false),
        }),
        "console_resize" => {
            let width = required_i64(table, "width", index, path)?;
            let height = required_i64(table, "height", index, path)?;
            if width <= 0 || height <= 0 {
                return Err(format!(
                    "Invalid `[[event]]` #{index} in `{}`: resize width and height must be positive (got {width}x{height}).",
                    path.display()
                ));
            }
            Ok(ScriptEvent::ConsoleResize { width, height })
        }
        "console_paste" => Ok(ScriptEvent::ConsolePaste {
            text: required_string(table, "paste", index, path)
                .or_else(|_| required_string(table, "text", index, path))?,
        }),
        "console_focus_gained" => Ok(ScriptEvent::ConsoleFocusGained),
        "console_focus_lost" => Ok(ScriptEvent::ConsoleFocusLost),
        "graph_key" => Ok(ScriptEvent::GraphKey {
            kind: required_string(table, "kind", index, path)?,
            ch: optional_char(table, "ch", index, path)?,
            shift: optional_bool(table, "shift").unwrap_or(false),
            ctrl: optional_bool(table, "ctrl").unwrap_or(false),
            alt: optional_bool(table, "alt").unwrap_or(false),
            meta: optional_bool(table, "meta").unwrap_or(false),
        }),
        "graph_mouse" => Ok(ScriptEvent::GraphMouse {
            action: required_string(table, "action", index, path)?,
            button: required_string(table, "button", index, path)?,
            x: required_i64(table, "x", index, path)?,
            y: required_i64(table, "y", index, path)?,
            shift: optional_bool(table, "shift").unwrap_or(false),
            ctrl: optional_bool(table, "ctrl").unwrap_or(false),
            alt: optional_bool(table, "alt").unwrap_or(false),
            meta: optional_bool(table, "meta").unwrap_or(false),
        }),
        "graph_wheel" => Ok(ScriptEvent::GraphWheel {
            delta_x: optional_i64(table, "delta_x").unwrap_or(0),
            delta_y: optional_i64(table, "delta_y").unwrap_or(0),
            shift: optional_bool(table, "shift").unwrap_or(false),
            ctrl: optional_bool(table, "ctrl").unwrap_or(false),
            alt: optional_bool(table, "alt").unwrap_or(false),
            meta: optional_bool(table, "meta").unwrap_or(false),
        }),
        other => Err(format!(
            "Unknown event type `{other}` in `[[event]]` #{index} of `{}`.\n  help: Supported types include `readln`, `console_key`, and `console_mouse`.",
            path.display()
        )),
    }
}

fn required_string(
    table: &toml::Table,
    field: &str,
    index: usize,
    path: &Path,
) -> Result<String, String> {
    let value = table.get(field).ok_or_else(|| {
        format!(
            "Missing `{field}` in `[[event]]` #{index} of `{}`.",
            path.display()
        )
    })?;
    value.as_str().map(str::to_string).ok_or_else(|| {
        format!(
            "Invalid `{field}` in `[[event]]` #{index} of `{}`: expected a string.",
            path.display()
        )
    })
}

fn required_i64(
    table: &toml::Table,
    field: &str,
    index: usize,
    path: &Path,
) -> Result<i64, String> {
    let value = table.get(field).ok_or_else(|| {
        format!(
            "Missing `{field}` in `[[event]]` #{index} of `{}`.",
            path.display()
        )
    })?;
    parse_i64_field(value).map_err(|error| {
        format!(
            "Invalid `{field}` in `[[event]]` #{index} of `{}`: {error}",
            path.display()
        )
    })
}

fn optional_i64(table: &toml::Table, field: &str) -> Option<i64> {
    table
        .get(field)
        .and_then(|value| parse_i64_field(value).ok())
}

fn optional_bool(table: &toml::Table, field: &str) -> Option<bool> {
    table
        .get(field)
        .and_then(|value| parse_bool_field(value).ok())
}

fn optional_char(
    table: &toml::Table,
    field: &str,
    index: usize,
    path: &Path,
) -> Result<Option<char>, String> {
    let Some(value) = table.get(field) else {
        return Ok(None);
    };
    let text = value.as_str().ok_or_else(|| {
        format!(
            "Invalid `{field}` in `[[event]]` #{index} of `{}`: expected a string.",
            path.display()
        )
    })?;
    let mut chars = text.chars();
    let ch = chars.next().ok_or_else(|| {
        format!(
            "Invalid `{field}` in `[[event]]` #{index} of `{}`: string must not be empty.",
            path.display()
        )
    })?;
    if chars.next().is_some() {
        return Err(format!(
            "Invalid `{field}` in `[[event]]` #{index} of `{}`: expected a single character.",
            path.display()
        ));
    }
    Ok(Some(ch))
}

fn parse_i64_field(value: &toml::Value) -> Result<i64, String> {
    match value {
        toml::Value::Integer(value) => Ok(*value),
        toml::Value::Float(value) if value.fract() == 0.0 => Ok(*value as i64),
        _ => Err("expected an integer".to_string()),
    }
}

fn parse_bool_field(value: &toml::Value) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| "expected true or false".to_string())
}
