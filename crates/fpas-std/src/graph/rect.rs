//! `Std.Graph` rectangle outline and fill primitives.
//!
//! **Documentation:** `docs/pascal/std/graph.md` (from the repository root).

use super::backbuffer::GraphBackbuffer;
use super::line;
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Draws one clipped rectangle outline into the runtime-owned backbuffer.
pub(crate) fn draw_rect(
    backbuffer: &mut GraphBackbuffer,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    color: u32,
    location: SourceLocation,
) -> Result<(), StdError> {
    let (width, height) = validate_rect_size(
        "Std.Graph.Application.DrawRect(App, X, Y, Width, Height, Color)",
        width,
        height,
        location,
    )?;

    if width == 1 || height == 1 {
        fill_rect(backbuffer, x, y, width, height, color, location)?;
        return Ok(());
    }

    let max_x = x + width - 1;
    let max_y = y + height - 1;
    line::draw_line(backbuffer, x, y, max_x, y, color);
    line::draw_line(backbuffer, x, max_y, max_x, max_y, color);
    line::draw_line(backbuffer, x, y, x, max_y, color);
    line::draw_line(backbuffer, max_x, y, max_x, max_y, color);
    Ok(())
}

/// Fills one clipped rectangle into the runtime-owned backbuffer.
pub(crate) fn fill_rect(
    backbuffer: &mut GraphBackbuffer,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    color: u32,
    location: SourceLocation,
) -> Result<(), StdError> {
    let (width, height) = validate_rect_size(
        "Std.Graph.Application.FillRect(App, X, Y, Width, Height, Color)",
        width,
        height,
        location,
    )?;

    for row in 0..height {
        for col in 0..width {
            backbuffer.put_pixel(x + col, y + row, color);
        }
    }
    Ok(())
}

fn validate_rect_size(
    operation: &str,
    width: i64,
    height: i64,
    location: SourceLocation,
) -> Result<(i64, i64), StdError> {
    if width <= 0 || height <= 0 {
        return Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "{operation} requires positive dimensions, got Width={width} and Height={height}."
            ),
            "Pass positive dimensions such as `Width=10` and `Height=5`.",
            location,
        ));
    }

    Ok((width, height))
}
