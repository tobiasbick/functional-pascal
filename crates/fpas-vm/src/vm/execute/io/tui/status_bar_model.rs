//! Decode Pascal status bar models from VM values.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::runtime_error;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{StatusBarSegment, StatusBarStyle, validate_packed_crt_color};

const STATUS_BAR_SEGMENT_TYPE: &str = "Std.Tui.StatusBarSegment";
const STATUS_BAR_STYLE_TYPE: &str = "Std.Tui.StatusBarStyle";

impl Worker {
    /// Parses `array of StatusBarSegment` from the stack top.
    pub(in crate::vm::execute::io::tui) fn pop_status_bar_segments(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<StatusBarSegment>, VmError> {
        match self.pop(line)? {
            Value::Array(values) => values
                .into_iter()
                .map(|value| self.decode_status_bar_segment(&value, line))
                .collect(),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Expected array of StatusBarSegment, got {}",
                    other.type_name()
                ),
                "Pass an array literal such as `[record Text := 'Ln 1'; AlignRight := false; end]`.",
                line,
            )),
        }
    }

    /// Parses `StatusBarStyle` from the stack top.
    pub(in crate::vm::execute::io::tui) fn pop_status_bar_style(
        &mut self,
        line: SourceLocation,
    ) -> Result<StatusBarStyle, VmError> {
        let value = self.pop(line)?;
        self.decode_status_bar_style(&value, line)
    }

    fn decode_status_bar_segment(
        &self,
        value: &Value,
        line: SourceLocation,
    ) -> Result<StatusBarSegment, VmError> {
        let Value::Record { type_name, fields } = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Expected {STATUS_BAR_SEGMENT_TYPE}, got {}",
                    value.type_name()
                ),
                "Each status segment must be a `StatusBarSegment` record.",
                line,
            ));
        };
        if type_name != STATUS_BAR_SEGMENT_TYPE {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {STATUS_BAR_SEGMENT_TYPE}, got `{type_name}`"),
                "Each status segment must be a `StatusBarSegment` record.",
                line,
            ));
        }

        let text = match Self::required_record_field(fields, "Text", line)? {
            Value::Str(text) => text.clone(),
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "StatusBarSegment.Text must be string, got {}",
                        other.type_name()
                    ),
                    "Set `Text := 'Ln 1, Col 1'` with a string literal.",
                    line,
                ));
            }
        };
        let align_right = match Self::required_record_field(fields, "AlignRight", line)? {
            Value::Boolean(flag) => *flag,
            other => {
                return Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "StatusBarSegment.AlignRight must be boolean, got {}",
                        other.type_name()
                    ),
                    "Set `AlignRight := true` for right-anchored hints.",
                    line,
                ));
            }
        };

        Ok(StatusBarSegment { text, align_right })
    }

    fn decode_status_bar_style(
        &self,
        value: &Value,
        line: SourceLocation,
    ) -> Result<StatusBarStyle, VmError> {
        let Value::Record { type_name, fields } = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Expected {STATUS_BAR_STYLE_TYPE}, got {}",
                    value.type_name()
                ),
                "Pass a `StatusBarStyle` record with CRT color indices.",
                line,
            ));
        };
        if type_name != STATUS_BAR_STYLE_TYPE {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {STATUS_BAR_STYLE_TYPE}, got `{type_name}`"),
                "Pass a `StatusBarStyle` record with CRT color indices.",
                line,
            ));
        }

        let bar_bg = validate_packed_crt_color(
            self.integer_record_field(fields, "BarBg", line)?,
            "StatusBarStyle.BarBg",
            line,
        )?;
        let bar_fg = validate_packed_crt_color(
            self.integer_record_field(fields, "BarFg", line)?,
            "StatusBarStyle.BarFg",
            line,
        )?;

        Ok(StatusBarStyle { bar_bg, bar_fg })
    }
}
