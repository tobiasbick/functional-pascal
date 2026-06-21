//! Decode FPAS control values.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{CommandId, RadioOption};

impl Worker {
    pub(super) fn pop_control_string(
        &mut self,
        label: &str,
        line: SourceLocation,
    ) -> Result<String, VmError> {
        match self.pop(line)? {
            Value::Str(value) => Ok(value),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("{label} must be string, got {}", other.type_name()),
                "Pass a string value.",
                line,
            )),
        }
    }

    pub(super) fn pop_radio_options(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<RadioOption>, VmError> {
        let Value::Array(values) = self.pop(line)? else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                "Options must be array of RadioOption",
                "Pass an array of RadioOption records.",
                line,
            ));
        };
        values
            .iter()
            .map(|value| self.decode_radio_option(value, line))
            .collect()
    }

    fn decode_radio_option(
        &self,
        value: &Value,
        line: SourceLocation,
    ) -> Result<RadioOption, VmError> {
        let Value::Record { type_name, fields } = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                "Expected Std.Tui.RadioOption record",
                "Build each option with `record label := ... end`.",
                line,
            ));
        };
        if type_name != "Std.Tui.RadioOption" {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected Std.Tui.RadioOption, got {type_name}"),
                "Pass RadioOption records.",
                line,
            ));
        }
        let label = match Self::required_record_field(fields, "label", line)? {
            Value::Str(v) => v.clone(),
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "RadioOption.label must be string, got {}",
                        other.type_name()
                    ),
                    "Set `label` to a string.",
                    line,
                ));
            }
        };
        let accelerator = optional_char(
            Self::required_record_field(fields, "accelerator", line)?,
            line,
        )?;
        let command_id = optional_integer(
            Self::required_record_field(fields, "commandId", line)?,
            line,
        )?
        .map(CommandId);
        let enabled = match Self::required_record_field(fields, "enabled", line)? {
            Value::Boolean(v) => *v,
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "RadioOption.enabled must be boolean, got {}",
                        other.type_name()
                    ),
                    "Set `enabled` to true or false.",
                    line,
                ));
            }
        };
        let mut option = RadioOption::new(label, accelerator, command_id);
        option.enabled = enabled;
        Ok(option)
    }
}

fn optional_char(value: &Value, line: SourceLocation) -> Result<Option<char>, VmError> {
    match value {
        Value::OptionNone => Ok(None),
        Value::OptionSome(v) => match &**v {
            Value::Char(ch) => Ok(Some(*ch)),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("accelerator must contain char, got {}", other.type_name()),
                "Pass None or Some('X').",
                line,
            )),
        },
        other => Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!(
                "accelerator must be Option of char, got {}",
                other.type_name()
            ),
            "Pass None or Some('X').",
            line,
        )),
    }
}

fn optional_integer(value: &Value, line: SourceLocation) -> Result<Option<i64>, VmError> {
    match value {
        Value::OptionNone => Ok(None),
        Value::OptionSome(v) => match &**v {
            Value::Integer(n) => Ok(Some(*n)),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("commandId must contain integer, got {}", other.type_name()),
                "Pass None or Some(CommandId).",
                line,
            )),
        },
        other => Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!(
                "commandId must be Option of integer, got {}",
                other.type_name()
            ),
            "Pass None or Some(CommandId).",
            line,
        )),
    }
}
