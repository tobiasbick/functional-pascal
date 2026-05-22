//! `Std.Graph` circle rasterization.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

use super::backbuffer::GraphBackbuffer;
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Draws one clipped circle outline into the runtime-owned backbuffer.
pub(crate) fn draw_circle(
    backbuffer: &mut GraphBackbuffer,
    center_x: i64,
    center_y: i64,
    radius: i64,
    color: u32,
    location: SourceLocation,
) -> Result<(), StdError> {
    if radius < 0 {
        return Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Std.Graph.Application.DrawCircle(App, CenterX, CenterY, Radius, Color) requires a non-negative radius, got Radius={radius}."
            ),
            "Pass `Radius >= 0`; `Radius = 0` draws a single pixel at the center.",
            location,
        ));
    }

    if radius == 0 {
        backbuffer.put_pixel(center_x, center_y, color);
        return Ok(());
    }

    let mut x = radius;
    let mut y = 0_i64;
    let mut error = 1 - radius;

    while x >= y {
        plot_circle_points(backbuffer, center_x, center_y, x, y, color);
        y += 1;
        if error < 0 {
            error += 2 * y + 1;
        } else {
            x -= 1;
            error += 2 * (y - x) + 1;
        }
    }

    Ok(())
}

fn plot_circle_points(
    backbuffer: &mut GraphBackbuffer,
    center_x: i64,
    center_y: i64,
    x: i64,
    y: i64,
    color: u32,
) {
    let points = [
        (center_x + x, center_y + y),
        (center_x + y, center_y + x),
        (center_x - y, center_y + x),
        (center_x - x, center_y + y),
        (center_x - x, center_y - y),
        (center_x - y, center_y - x),
        (center_x + y, center_y - x),
        (center_x + x, center_y - y),
    ];

    for (x, y) in points {
        backbuffer.put_pixel(x, y, color);
    }
}
