//! Turbo Vision geometry helpers shared across bridge modules.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::shared::TurboVisionRect;

pub(super) fn state_rect(rect: turbo_vision::core::geometry::Rect) -> TurboVisionRect {
    TurboVisionRect {
        x: rect.a.x,
        y: rect.a.y,
        width: rect.b.x - rect.a.x,
        height: rect.b.y - rect.a.y,
    }
}

pub(super) fn turbo_rect(rect: TurboVisionRect) -> turbo_vision::core::geometry::Rect {
    turbo_vision::core::geometry::Rect::from_coords(rect.x, rect.y, rect.width, rect.height)
}
