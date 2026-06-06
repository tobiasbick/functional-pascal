use super::Console;
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Console {
    /// Returns `Some(coord)` if `raw` is a valid 1-based coordinate within `[1, max]`;
    /// returns `None` when the coordinate is out of bounds so the caller can silently skip
    /// the operation (e.g. after a terminal resize).
    pub(super) fn check_relative_coord(&self, raw: i64, max: u16) -> Option<u16> {
        let value = u16::try_from(raw).ok()?;
        if value == 0 || value > max {
            None
        } else {
            Some(value)
        }
    }

    /// Returns `Some(coord)` if `raw` is a valid 1-based absolute screen coordinate within
    /// `[1, max]`; returns `None` when out of bounds so the caller can silently skip.
    pub(super) fn check_absolute_coord(&self, raw: i64, max: u16) -> Option<u16> {
        let value = u16::try_from(raw).ok()?;
        if value == 0 || value > max {
            None
        } else {
            Some(value)
        }
    }

    pub(super) fn validate_color(
        &self,
        raw: i64,
        op_name: &str,
        location: SourceLocation,
    ) -> Result<u8, StdError> {
        validate_packed_crt_color(raw, op_name, location)
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

    pub(super) fn validate_color_256(
        &self,
        raw: i64,
        op_name: &str,
        location: SourceLocation,
    ) -> Result<u8, StdError> {
        if !(0..=255).contains(&raw) {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("{op_name} expects a color index from 0 to 255, got {raw}"),
                "Pass an integer from 0 to 255 for the 256-color palette.",
                location,
            ));
        }
        Ok(raw as u8)
    }

    pub(super) fn validate_rgb(
        &self,
        r: i64,
        g: i64,
        b: i64,
        op_name: &str,
        location: SourceLocation,
    ) -> Result<(u8, u8, u8), StdError> {
        for (ch, val) in [("R", r), ("G", g), ("B", b)] {
            if !(0..=255).contains(&val) {
                return Err(std_runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("{op_name} expects {ch} in 0–255, got {val}"),
                    "Each channel (R, G, B) must be an integer from 0 to 255.",
                    location,
                ));
            }
        }
        Ok((r as u8, g as u8, b as u8))
    }
}

/// Validate a packed CRT color index (`0..=15`).
pub fn validate_packed_crt_color(
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
