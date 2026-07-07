//! FPAS `Rect` ↔ turbo-vision `Rect` conversion for try-2.
//!
//! **Documentation:** `docs/refactor-tui-try-2/upstream-mapping.md`

/// FPAS width/height rectangle to upstream corner `Rect`.
#[must_use]
pub fn fpas_rect_to_turbo_vision(
    x: i16,
    y: i16,
    width: i16,
    height: i16,
) -> turbo_vision::core::geometry::Rect {
    turbo_vision::core::geometry::Rect::from_coords(x, y, width, height)
}

/// Upstream corner `Rect` to FPAS width/height fields.
#[must_use]
pub fn turbo_vision_rect_to_fpas(rect: turbo_vision::core::geometry::Rect) -> (i16, i16, i16, i16) {
    (rect.a.x, rect.a.y, rect.b.x - rect.a.x, rect.b.y - rect.a.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_rect() {
        let tv = fpas_rect_to_turbo_vision(4, 2, 40, 13);
        assert_eq!(turbo_vision_rect_to_fpas(tv), (4, 2, 40, 13));
    }
}
