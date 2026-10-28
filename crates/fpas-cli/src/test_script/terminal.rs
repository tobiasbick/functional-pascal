//! Structured terminal input in test and batch-debugger scripts.

use fpas_runtime::execution::{DebugTerminalEvent, DebugTerminalKeyEvent, DebugTerminalKeyKind};

pub(super) fn key(table: &toml::Table) -> Result<DebugTerminalEvent, String> {
    validate_fields(
        table,
        &["type", "kind", "ch", "shift", "ctrl", "alt", "meta"],
    )?;
    let name = required_string(table, "kind")?;
    let kind = if name == "Character" {
        let text = required_string(table, "ch")?;
        let mut characters = text.chars();
        let character = characters
            .next()
            .filter(|_| characters.next().is_none())
            .ok_or("`ch` must contain exactly one Unicode scalar for `Character`")?;
        DebugTerminalKeyKind::Character(character)
    } else {
        if table.contains_key("ch") {
            return Err("`ch` is only supported for `kind = \"Character\"`".to_string());
        }
        DebugTerminalKeyKind::named(name)
            .ok_or_else(|| format!("unsupported key `kind` `{name}`"))?
    };
    Ok(DebugTerminalEvent::Key(DebugTerminalKeyEvent {
        kind,
        shift: flag(table, "shift")?,
        ctrl: flag(table, "ctrl")?,
        alt: flag(table, "alt")?,
        meta: flag(table, "meta")?,
    }))
}

pub(super) fn resize(table: &toml::Table) -> Result<DebugTerminalEvent, String> {
    validate_fields(table, &["type", "width", "height"])?;
    Ok(DebugTerminalEvent::Resize {
        width: dimension(table, "width")?,
        height: dimension(table, "height")?,
    })
}

fn validate_fields(table: &toml::Table, allowed: &[&str]) -> Result<(), String> {
    if let Some(field) = table
        .keys()
        .find(|field| !allowed.contains(&field.as_str()))
    {
        return Err(format!("unknown field `{field}`"));
    }
    Ok(())
}

fn required_string<'a>(table: &'a toml::Table, field: &str) -> Result<&'a str, String> {
    table
        .get(field)
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("`{field}` must be a string"))
}

fn flag(table: &toml::Table, field: &str) -> Result<bool, String> {
    match table.get(field) {
        None => Ok(false),
        Some(value) => value
            .as_bool()
            .ok_or_else(|| format!("`{field}` must be a boolean")),
    }
}

fn dimension(table: &toml::Table, field: &str) -> Result<u16, String> {
    table
        .get(field)
        .and_then(toml::Value::as_integer)
        .and_then(|value| u16::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("`{field}` must be an integer in 1..=65535"))
}
