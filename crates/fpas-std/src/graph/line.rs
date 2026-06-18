//! `Std.Graph` line rasterization.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md` (from the repository root).

use super::backbuffer::GraphBackbuffer;

/// Draws one clipped line into the runtime-owned backbuffer.
pub(crate) fn draw_line(
    backbuffer: &mut GraphBackbuffer,
    x1: i64,
    y1: i64,
    x2: i64,
    y2: i64,
    color: u32,
) {
    let mut x = i128::from(x1);
    let mut y = i128::from(y1);
    let target_x = i128::from(x2);
    let target_y = i128::from(y2);
    let dx = (target_x - x).abs();
    let dy = -((target_y - y).abs());
    let step_x = if x < target_x { 1 } else { -1 };
    let step_y = if y < target_y { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        if let (Ok(x), Ok(y)) = (i64::try_from(x), i64::try_from(y)) {
            backbuffer.put_pixel(x, y, color);
        }

        if x == target_x && y == target_y {
            break;
        }

        let doubled_error = error.saturating_mul(2);
        if doubled_error >= dy {
            error += dy;
            x += step_x;
        }
        if doubled_error <= dx {
            error += dx;
            y += step_y;
        }
    }
}
