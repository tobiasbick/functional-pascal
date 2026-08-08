//! Runtime conversion for `Std.Console` cell, color, rectangle, and region records.

use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_CONSOLE_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH};
use fpas_std::{ConsoleCell, ConsoleColor, ConsoleRect, SavedRegionId};

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, internal_error, runtime_error};

const COLOR_TYPE: &str = "Std.Console.Color";
const CELL_TYPE: &str = "Std.Console.Cell";
const RECT_TYPE: &str = "Std.Console.Rect";

impl Worker {
    /// Builds one FPAS `Std.Console.Color` record.
    pub(crate) fn console_color_record(
        &self,
        color: ConsoleColor,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let (kind, index, red, green, blue) = match color {
            ConsoleColor::Crt(index) => (0, i64::from(index), 0, 0, 0),
            ConsoleColor::Ansi256(index) => (1, i64::from(index), 0, 0, 0),
            ConsoleColor::Rgb { red, green, blue } => {
                (2, 0, i64::from(red), i64::from(green), i64::from(blue))
            }
        };
        self.record_value(
            COLOR_TYPE,
            vec![
                Value::Integer(kind),
                Value::Integer(index),
                Value::Integer(red),
                Value::Integer(green),
                Value::Integer(blue),
            ],
            location,
        )
    }
    /// Validates and constructs a classic CRT palette color.
    pub(crate) fn console_crt_color(
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
    pub(crate) fn console_ansi256_color(
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
    pub(crate) fn console_rgb_color(
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
    pub(crate) fn console_cell_record(
        &self,
        cell: ConsoleCell,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        self.record_value(
            CELL_TYPE,
            vec![
                Value::Str(cell.glyph.into()),
                self.console_color_record(cell.foreground, location)?,
                self.console_color_record(cell.background, location)?,
            ],
            location,
        )
    }

    /// Builds an opaque FPAS `Std.Console.SavedRegion` handle record.
    pub(crate) fn saved_region_record(
        &self,
        id: SavedRegionId,
        _location: SourceLocation,
    ) -> Result<Value, VmError> {
        Ok(Value::OpaqueHandle(id.0))
    }
}

pub(crate) fn console_cell_from_value(
    value: &Value,
    line: SourceLocation,
) -> Result<ConsoleCell, VmError> {
    let glyph = match field(value, "glyph", CELL_TYPE, line)? {
        Value::Str(glyph) => glyph.to_string(),
        other => {
            return Err(field_type_error(CELL_TYPE, "glyph", "string", other, line));
        }
    };
    let foreground = console_color_from_value(field(value, "foreground", CELL_TYPE, line)?, line)?;
    let background = console_color_from_value(field(value, "background", CELL_TYPE, line)?, line)?;
    Ok(ConsoleCell {
        glyph,
        foreground,
        background,
    })
}

fn console_color_from_value(value: &Value, line: SourceLocation) -> Result<ConsoleColor, VmError> {
    let kind = integer_field(value, "kind", COLOR_TYPE, line)?;
    match kind {
        0 => Ok(ConsoleColor::Crt(integer_u8_field(
            value, "index", 15, COLOR_TYPE, line,
        )?)),
        1 => Ok(ConsoleColor::Ansi256(integer_u8_field(
            value, "index", 255, COLOR_TYPE, line,
        )?)),
        2 => Ok(ConsoleColor::Rgb {
            red: integer_u8_field(value, "red", 255, COLOR_TYPE, line)?,
            green: integer_u8_field(value, "green", 255, COLOR_TYPE, line)?,
            blue: integer_u8_field(value, "blue", 255, COLOR_TYPE, line)?,
        }),
        _ => Err(validation_error(
            format!("Std.Console.Color.kind must be Crt, Ansi256, or Rgb, got {kind}"),
            "Use CrtColor, Ansi256Color, or RgbColor to construct colors.",
            line,
        )),
    }
}

pub(crate) fn console_rect_from_value(
    value: &Value,
    line: SourceLocation,
) -> Result<ConsoleRect, VmError> {
    Ok(ConsoleRect {
        x: positive_u16_field(value, "x", RECT_TYPE, line)?,
        y: positive_u16_field(value, "y", RECT_TYPE, line)?,
        width: positive_u16_field(value, "width", RECT_TYPE, line)?,
        height: positive_u16_field(value, "height", RECT_TYPE, line)?,
    })
}

fn field<'a>(
    value: &'a Value,
    name: &str,
    record: &str,
    line: SourceLocation,
) -> Result<&'a Value, VmError> {
    match value {
        Value::Record(value) => value
            .body()
            .layout
            .fields
            .iter()
            .position(|field_name| field_name == name)
            .and_then(|index| value.body().values.get(index)),
        other => {
            return Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected {record}, got {}", other.type_name()),
                format!("Pass a `{record}` value."),
                line,
            ));
        }
    }
    .ok_or_else(|| {
        internal_error(
            format!("{record} is missing field `{name}`"),
            "This indicates a compiler/runtime mismatch.",
            line,
        )
    })
}

fn integer_field(
    value: &Value,
    name: &str,
    record: &str,
    line: SourceLocation,
) -> Result<i64, VmError> {
    match field(value, name, record, line)? {
        Value::Integer(value) => Ok(*value),
        other => Err(field_type_error(record, name, "integer", other, line)),
    }
}

fn integer_u8_field(
    record_value: &Value,
    name: &str,
    max: u8,
    record: &str,
    line: SourceLocation,
) -> Result<u8, VmError> {
    let value = integer_field(record_value, name, record, line)?;
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
    record_value: &Value,
    name: &str,
    record: &str,
    line: SourceLocation,
) -> Result<u16, VmError> {
    let value = integer_field(record_value, name, record, line)?;
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
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
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

pub(crate) fn console_color_record(
    worker: &Worker,
    color: ConsoleColor,
    location: SourceLocation,
) -> Result<Value, VmError> {
    worker.console_color_record(color, location)
}

pub(crate) fn console_crt_color(index: i64, line: SourceLocation) -> Result<ConsoleColor, VmError> {
    Worker::console_crt_color(index, line)
}

pub(crate) fn console_ansi256_color(
    index: i64,
    line: SourceLocation,
) -> Result<ConsoleColor, VmError> {
    Worker::console_ansi256_color(index, line)
}

pub(crate) fn console_rgb_color(
    red: i64,
    green: i64,
    blue: i64,
    line: SourceLocation,
) -> Result<ConsoleColor, VmError> {
    Worker::console_rgb_color(red, green, blue, line)
}

pub(crate) fn console_cell_record(
    worker: &Worker,
    cell: ConsoleCell,
    location: SourceLocation,
) -> Result<Value, VmError> {
    worker.console_cell_record(cell, location)
}

pub(crate) fn saved_region_record(
    worker: &Worker,
    id: SavedRegionId,
    location: SourceLocation,
) -> Result<Value, VmError> {
    worker.saved_region_record(id, location)
}

pub(crate) fn saved_region_from_value(
    value: &Value,
    line: SourceLocation,
) -> Result<SavedRegionId, VmError> {
    let id = match value {
        Value::OpaqueHandle(id) if *id != 0 => *id,
        _ => {
            return Err(validation_error(
                "Std.Console.SavedRegion contains an invalid handle",
                "Use the SavedRegion returned by SaveRegion.",
                line,
            ));
        }
    };
    Ok(SavedRegionId(id))
}
