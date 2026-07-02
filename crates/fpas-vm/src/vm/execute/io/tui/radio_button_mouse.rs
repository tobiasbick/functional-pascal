//! Mouse click handling for Turbo Vision radio buttons.
//!
//! Upstream `turbo_vision::RadioButton` selects on Space when focused only. FPAS mirrors
//! Borland-style single-click select on left mouse down inside the control bounds.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use turbo_vision::core::event::{Event, EventType, MB_LEFT_BUTTON};
use turbo_vision::views::radiobutton::RadioButton;
use turbo_vision::views::view::View;

/// Select `radio_button` when the event is a left-button mouse down inside its bounds.
///
/// Returns `true` when the event was consumed.
pub(in crate::vm::execute::io::tui) fn try_select_radio_button_on_mouse_down(
    radio_button: &mut RadioButton,
    event: &mut Event,
) -> bool {
    if event.what != EventType::MouseDown {
        return false;
    }
    if event.mouse.buttons & MB_LEFT_BUTTON == 0 {
        return false;
    }
    if !radio_button.bounds().contains(event.mouse.pos) {
        return false;
    }

    radio_button.select();
    event.clear();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::event::Event;
    use turbo_vision::core::geometry::{Point, Rect};

    #[test]
    fn mouse_down_inside_bounds_selects_radio_button() {
        let bounds = Rect::new(2, 3, 12, 3);
        let mut radio_button = RadioButton::new(bounds, "opt", 1);
        assert!(!radio_button.is_selected());

        let mut event = Event::mouse(EventType::MouseDown, bounds.a, MB_LEFT_BUTTON, false);
        assert!(try_select_radio_button_on_mouse_down(
            &mut radio_button,
            &mut event
        ));
        assert!(radio_button.is_selected());
        assert_eq!(event.what, EventType::Nothing);
    }

    #[test]
    fn mouse_down_outside_bounds_is_ignored() {
        let bounds = Rect::new(2, 3, 12, 3);
        let mut radio_button = RadioButton::new(bounds, "opt", 1);

        let mut event = Event::mouse(
            EventType::MouseDown,
            Point::new(bounds.b.x, bounds.a.y),
            MB_LEFT_BUTTON,
            false,
        );
        assert!(!try_select_radio_button_on_mouse_down(
            &mut radio_button,
            &mut event
        ));
        assert!(!radio_button.is_selected());
        assert_eq!(event.what, EventType::MouseDown);
    }
}
