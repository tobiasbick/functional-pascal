//! Runtime conversion for `Std.Console` cell, color, rectangle, and region records.

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::{ConsoleCell, ConsoleColor, ConsoleRect, SavedRegionId};

const COLOR_TYPE: &str = "Std.Console.Color";
const CELL_TYPE: &str = "Std.Console.Cell";
const RECT_TYPE: &str = "Std.Console.Rect";
const SAVED_REGION_TYPE: &str = "Std.Console.SavedRegion";
const HANDLE_FIELD: &str = "__id";

impl Worker {
    /// Builds one FPAS `Std.Console.Color` record.
    pub(in crate::vm::execute::io) fn console_color_record(color: ConsoleColor) -> Value {
        let (kind, index, red, green, blue) = match color {
            ConsoleColor::Crt(index) => (0, i64::from(index), 0, 0, 0),
            ConsoleColor::Ansi256(index) => (1, i64::from(index), 0, 0, 0),
            ConsoleColor::Rgb { red, green, blue } => {
                (2, 0, i64::from(red), i64::from(green), i64::from(blue))
            }
        };
        Value::Record {
            type_name: COLOR_TYPE.into(),
            fields: vec![
                ("kind".into(), Value::Integer(kind)),
                ("index".into(), Value::Integer(index)),
                ("red".into(), Value::Integer(red)),
                ("green".into(), Value::Integer(green)),
                ("blue".into(), Value::Integer(blue)),
            ],
        }
    }

    /// Validates and constructs a classic CRT palette color.
    pub(in crate::vm::execute::io) fn console_crt_color(
        index: i64,
        line: SourceLocation,
    ) -> Result<ConsoleColor, VmError> {
        Ok(ConsoleColor::Crt(integer_to_u8(
            index,
            15,
            "CrtColor.Index",
            line,
        )?))
    }

    /// Validates and constructs an ANSI 256-palette color.
    pub(in crate::vm::execute::io) fn console_ansi256_color(
        index: i64,
        line: SourceLocation,
    ) -> Result<ConsoleColor, VmError> {
        Ok(ConsoleColor::Ansi256(integer_to_u8(
            index,
            255,
            "Ansi256Color.Index",
            line,
        )?))
    }

    /// Validates and constructs a truecolor value.
    pub(in crate::vm::execute::io) fn console_rgb_color(
        red: i64,
        green: i64,
        blue: i64,
        line: SourceLocation,
    ) -> Result<ConsoleColor, VmError> {
        Ok(ConsoleColor::Rgb {
            red: integer_to_u8(red, 255, "RgbColor.Red", line)?,
            green: integer_to_u8(green, 255, "RgbColor.Green", line)?,
            blue: integer_to_u8(blue, 255, "RgbColor.Blue", line)?,
        })
    }

    /// Builds one FPAS `Std.Console.Cell` record.
    pub(in crate::vm::execute::io) fn console_cell_record(cell: ConsoleCell) -> Value {
        Value::Record {
            type_name: CELL_TYPE.into(),
            fields: vec![
                ("glyph".into(), Value::Str(cell.glyph.to_string())),
                (
                    "foreground".into(),
                    Self::console_color_record(cell.foreground),
                ),
                (
                    "background".into(),
                    Self::console_color_record(cell.background),
                ),
            ],
        }
    }

    /// Pops and validates one FPAS `Std.Console.Cell` record.
    pub(in crate::vm::execute::io) fn pop_console_cell(
        &mut self,
        line: SourceLocation,
    ) -> Result<ConsoleCell, VmError> {
        let value = self.pop(line)?;
        console_cell_from_value(&value, line)
    }

    /// Pops and validates one FPAS `Std.Console.Rect` record.
    pub(in crate::vm::execute::io) fn pop_console_rect(
        &mut self,
        line: SourceLocation,
    ) -> Result<ConsoleRect, VmError> {
        let value = self.pop(line)?;
        console_rect_from_value(&value, line)
    }

    /// Pops and validates an array of FPAS `Std.Console.Cell` records.
    pub(in crate::vm::execute::io) fn pop_console_cells(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<ConsoleCell>, VmError> {
        match self.pop(line)? {
            Value::Array(values) => values
                .iter()
                .map(|value| console_cell_from_value(value, line))
                .collect(),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Expected array of Std.Console.Cell, got {}",
                    other.type_name()
                ),
                "Pass an `array of Cell` as the `Values` argument.",
                line,
            )),
        }
    }

    /// Pops one string argument for a `Std.Console` operation.
    pub(in crate::vm::execute::io) fn pop_console_text(
        &mut self,
        line: SourceLocation,
    ) -> Result<String, VmError> {
        match self.pop(line)? {
            Value::Str(text) => Ok(text),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected string, got {}", other.type_name()),
                "Pass a string as the `Text` argument.",
                line,
            )),
        }
    }

    /// Builds an opaque FPAS `Std.Console.SavedRegion` handle record.
    pub(in crate::vm::execute::io) fn saved_region_record(id: SavedRegionId) -> Value {
        Value::Record {
            type_name: SAVED_REGION_TYPE.into(),
            fields: vec![(HANDLE_FIELD.into(), Value::Integer(id.0 as i64))],
        }
    }

    /// Pops and validates an opaque FPAS `Std.Console.SavedRegion` handle.
    pub(in crate::vm::execute::io) fn pop_saved_region(
        &mut self,
        line: SourceLocation,
    ) -> Result<SavedRegionId, VmError> {
        let value = self.pop(line)?;
        let fields = record_fields(&value, SAVED_REGION_TYPE, line)?;
        let id = integer_field(fields, HANDLE_FIELD, SAVED_REGION_TYPE, line)?;
        let id = u64::try_from(id)
            .ok()
            .filter(|id| *id != 0)
            .ok_or_else(|| {
                validation_error(
                    "Std.Console.SavedRegion contains an invalid handle",
                    "Use the SavedRegion returned by SaveRegion.",
                    line,
                )
            })?;
        Ok(SavedRegionId(id))
    }
}

fn console_cell_from_value(value: &Value, line: SourceLocation) -> Result<ConsoleCell, VmError> {
    let fields = record_fields(value, CELL_TYPE, line)?;
    let glyph = match field(fields, "glyph", CELL_TYPE, line)? {
        Value::Str(glyph) => {
            let mut chars = glyph.chars();
            match (chars.next(), chars.next()) {
                (Some(glyph), None) => glyph,
                _ => {
                    return Err(validation_error(
                        "Std.Console.Cell.glyph must contain exactly one Unicode scalar",
                        "Set `glyph` to a string containing one character.",
                        line,
                    ));
                }
            }
        }
        other => {
            return Err(field_type_error(CELL_TYPE, "glyph", "string", other, line));
        }
    };
    let foreground = console_color_from_value(field(fields, "foreground", CELL_TYPE, line)?, line)?;
    let background = console_color_from_value(field(fields, "background", CELL_TYPE, line)?, line)?;
    Ok(ConsoleCell {
        glyph,
        foreground,
        background,
    })
}

fn console_color_from_value(value: &Value, line: SourceLocation) -> Result<ConsoleColor, VmError> {
    let fields = record_fields(value, COLOR_TYPE, line)?;
    let kind = integer_field(fields, "kind", COLOR_TYPE, line)?;
    match kind {
        0 => Ok(ConsoleColor::Crt(integer_u8_field(
            fields, "index", 15, COLOR_TYPE, line,
        )?)),
        1 => Ok(ConsoleColor::Ansi256(integer_u8_field(
            fields, "index", 255, COLOR_TYPE, line,
        )?)),
        2 => Ok(ConsoleColor::Rgb {
            red: integer_u8_field(fields, "red", 255, COLOR_TYPE, line)?,
            green: integer_u8_field(fields, "green", 255, COLOR_TYPE, line)?,
            blue: integer_u8_field(fields, "blue", 255, COLOR_TYPE, line)?,
        }),
        _ => Err(validation_error(
            format!("Std.Console.Color.kind must be Crt, Ansi256, or Rgb, got {kind}"),
            "Use CrtColor, Ansi256Color, or RgbColor to construct colors.",
            line,
        )),
    }
}

fn console_rect_from_value(value: &Value, line: SourceLocation) -> Result<ConsoleRect, VmError> {
    let fields = record_fields(value, RECT_TYPE, line)?;
    Ok(ConsoleRect {
        x: positive_u16_field(fields, "x", RECT_TYPE, line)?,
        y: positive_u16_field(fields, "y", RECT_TYPE, line)?,
        width: positive_u16_field(fields, "width", RECT_TYPE, line)?,
        height: positive_u16_field(fields, "height", RECT_TYPE, line)?,
    })
}

fn record_fields<'a>(
    value: &'a Value,
    expected: &str,
    line: SourceLocation,
) -> Result<&'a [(String, Value)], VmError> {
    match value {
        Value::Record { type_name, fields } if type_name == expected || type_name == "<record>" => {
            Ok(fields)
        }
        other => Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!("Expected {expected}, got {}", other.type_name()),
            format!("Pass a `{expected}` value."),
            line,
        )),
    }
}

fn field<'a>(
    fields: &'a [(String, Value)],
    name: &str,
    record: &str,
    line: SourceLocation,
) -> Result<&'a Value, VmError> {
    fields
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
        .ok_or_else(|| {
            internal_error(
                format!("{record} is missing field `{name}`"),
                "This indicates a compiler/runtime mismatch.",
                line,
            )
        })
}

fn integer_field(
    fields: &[(String, Value)],
    name: &str,
    record: &str,
    line: SourceLocation,
) -> Result<i64, VmError> {
    match field(fields, name, record, line)? {
        Value::Integer(value) => Ok(*value),
        other => Err(field_type_error(record, name, "integer", other, line)),
    }
}

fn integer_u8_field(
    fields: &[(String, Value)],
    name: &str,
    max: u8,
    record: &str,
    line: SourceLocation,
) -> Result<u8, VmError> {
    let value = integer_field(fields, name, record, line)?;
    u8::try_from(value)
        .ok()
        .filter(|value| *value <= max)
        .ok_or_else(|| {
            validation_error(
                format!("{record}.{name} must be in 0..={max}, got {value}"),
                "Use a color component within the documented range.",
                line,
            )
        })
}

fn integer_to_u8(value: i64, max: u8, label: &str, line: SourceLocation) -> Result<u8, VmError> {
    u8::try_from(value)
        .ok()
        .filter(|value| *value <= max)
        .ok_or_else(|| {
            validation_error(
                format!("Std.Console.{label} must be in 0..={max}, got {value}"),
                "Use a color component within the documented range.",
                line,
            )
        })
}

fn positive_u16_field(
    fields: &[(String, Value)],
    name: &str,
    record: &str,
    line: SourceLocation,
) -> Result<u16, VmError> {
    let value = integer_field(fields, name, record, line)?;
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            validation_error(
                format!("{record}.{name} must be in 1..=65535, got {value}"),
                "Use positive 1-based coordinates and positive rectangle sizes.",
                line,
            )
        })
}

fn field_type_error(
    record: &str,
    field: &str,
    expected: &str,
    actual: &Value,
    line: SourceLocation,
) -> VmError {
    runtime_error(
        TYPE_MISMATCH_CODE,
        format!(
            "{record}.{field} must be {expected}, got {}",
            actual.type_name()
        ),
        format!("Construct `{record}` with the declared field types."),
        line,
    )
}

fn validation_error(
    message: impl Into<String>,
    help: impl Into<String>,
    line: SourceLocation,
) -> VmError {
    runtime_error(RUNTIME_CONSOLE_STATE_ERROR, message, help, line)
}
