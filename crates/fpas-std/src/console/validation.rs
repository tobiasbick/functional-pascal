use super::Console;
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Console {
    pub(super) fn validate_relative_coord(
        &self,
        raw: i64,
        max: u16,
        name: &str,
        location: SourceLocation,
    ) -> Result<u16, StdError> {
        let Ok(value) = u16::try_from(raw) else {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{name} coordinate {raw} is out of range"),
                format!("Use a value between 1 and {max} inside the active console window."),
                location,
            ));
        };
        if value == 0 || value > max {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{name} coordinate {raw} is outside the active window"),
                format!("Use a value between 1 and {max} inside the active console window."),
                location,
            ));
        }
        Ok(value)
    }

    pub(super) fn validate_absolute_coord(
        &self,
        raw: i64,
        max: u16,
        name: &str,
        location: SourceLocation,
    ) -> Result<u16, StdError> {
        let Ok(value) = u16::try_from(raw) else {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{name} coordinate {raw} is out of range"),
                format!("Use a value between 1 and {max} on the current screen."),
                location,
            ));
        };
        if value == 0 || value > max {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{name} coordinate {raw} is outside the screen"),
                format!("Use a value between 1 and {max} on the current screen."),
                location,
            ));
        }
        Ok(value)
    }

    pub(super) fn validate_color(
        &self,
        raw: i64,
        op_name: &str,
        location: SourceLocation,
    ) -> Result<u8, StdError> {
        if !(0..=15).contains(&raw) {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{op_name} expects a color index from 0 to 15, got {raw}"),
                "Use one of the CRT color constants such as `LightRed` or an integer from 0 to 15.",
                location,
            ));
        }
        Ok(raw as u8)
    }

    pub(super) fn validate_text_attr(
        &self,
        raw: i64,
        location: SourceLocation,
    ) -> Result<u8, StdError> {
        if !(0..=255).contains(&raw) {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("SetTextAttr expects an attribute from 0 to 255, got {raw}"),
                "Use `TextAttr` values encoded as (Background * 16 + Foreground).",
                location,
            ));
        }
        Ok(raw as u8)
    }
}
