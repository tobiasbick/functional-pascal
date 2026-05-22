//! `Std.Graph` packed RGB24 validation helpers.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Highest valid packed `$00RRGGBB` color value accepted by `Std.Graph`.
pub(crate) const MAX_RGB24: i64 = 0x00FF_FFFF;

/// Validates one packed `$00RRGGBB` color and returns it as `u32`.
pub(crate) fn validate_rgb24(
    color: i64,
    operation: &str,
    location: SourceLocation,
) -> Result<u32, StdError> {
    if !(0..=MAX_RGB24).contains(&color) {
        return Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("{operation} requires `$00RRGGBB` colors; got {color}."),
            "Pass an integer between 0 and 16777215 (`$00RRGGBB`).",
            location,
        ));
    }

    Ok(u32::try_from(color).unwrap_or_default())
}
