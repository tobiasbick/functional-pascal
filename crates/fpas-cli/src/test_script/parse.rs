//! TOML sidecar parsing for test scripts.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::fs;
use std::path::{Path, PathBuf};

/// One scripted input event before `vm.run()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptEvent {
    Readln { line: String },
    ReadkeyChars { chars: String },
}

/// Parsed sidecar script contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptFile {
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
            "Invalid script `{}`: {error}\n  help: See docs/pascal/std/testing/test.md.",
            path.display()
        )
    })?;

    if let Some(key) = root.keys().find(|key| key.as_str() != "event") {
        return Err(format!(
            "Unknown script section or field `{key}` in `{}`.\n  help: Sidecar scripts only support `[[event]]` entries.",
            path.display()
        ));
    }

    let events = parse_events(root.get("event"), path)?;
    Ok(ScriptFile { events })
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
        other => Err(format!(
            "Unknown event type `{other}` in `[[event]]` #{index} of `{}`.\n  help: Supported types are `readln` and `readkey_chars`.",
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
