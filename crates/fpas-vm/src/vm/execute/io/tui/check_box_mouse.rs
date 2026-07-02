//! Mouse click handling for Turbo Vision check boxes.
//!
//! Upstream `turbo_vision::CheckBox` toggles on Space when focused only. FPAS mirrors
//! Borland-style single-click toggle on left mouse down inside the control bounds.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use turbo_vision::core::event::{Event, EventType, MB_LEFT_BUTTON};
use turbo_vision::views::checkbox::CheckBox;
use turbo_vision::views::view::View;

/// Toggle `check_box` when the event is a left-button mouse down inside its bounds.
///
/// Returns `true` when the event was consumed.
pub(in crate::vm::execute::io::tui) fn try_toggle_check_box_on_mouse_down(
    check_box: &mut CheckBox,
    event: &mut Event,
) -> bool {
    if event.what != EventType::MouseDown {
        return false;
    }
    if event.mouse.buttons & MB_LEFT_BUTTON == 0 {
        return false;
    }
    if !check_box.bounds().contains(event.mouse.pos) {
        return false;
    }

    check_box.toggle();
    event.clear();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::event::Event;
    use turbo_vision::core::geometry::{Point, Rect};

    #[test]
    fn mouse_down_inside_bounds_toggles_check_box() {
        let bounds = Rect::new(2, 3, 12, 3);
        let mut check_box = CheckBox::new(bounds, "opt");
        assert!(!check_box.is_checked());

        let mut event = Event::mouse(
            EventType::MouseDown,
            bounds.a,
            MB_LEFT_BUTTON,
            false,
        );
        assert!(try_toggle_check_box_on_mouse_down(
            &mut check_box,
            &mut event
        ));
        assert!(check_box.is_checked());
        assert_eq!(event.what, EventType::Nothing);
    }

    #[test]
    fn mouse_down_outside_bounds_is_ignored() {
        let bounds = Rect::new(2, 3, 12, 3);
        let mut check_box = CheckBox::new(bounds, "opt");

        let mut event = Event::mouse(
            EventType::MouseDown,
            Point::new(bounds.b.x, bounds.a.y),
            MB_LEFT_BUTTON,
            false,
        );
        assert!(!try_toggle_check_box_on_mouse_down(
            &mut check_box,
            &mut event
        ));
        assert!(!check_box.is_checked());
        assert_eq!(event.what, EventType::MouseDown);
    }
}
