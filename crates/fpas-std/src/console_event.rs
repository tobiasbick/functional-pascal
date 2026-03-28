//! `Std.Console` event model for later TUI-style input handling.
//!
//! Spec: `docs/pascal/std/console.md` (from repository root).

use crate::key_event::{ConsoleKeyEvent, key_kind_index};

pub const EVENT_KIND_VARIANTS: &[&str] = &[
    "Key",
    "Mouse",
    "Resize",
    "Paste",
    "FocusGained",
    "FocusLost",
];

pub const MOUSE_ACTION_VARIANTS: &[&str] = &[
    "Unknown",
    "Down",
    "Up",
    "Drag",
    "Move",
    "ScrollDown",
    "ScrollUp",
    "ScrollLeft",
    "ScrollRight",
];

pub const MOUSE_BUTTON_VARIANTS: &[&str] = &["None", "Left", "Right", "Middle"];

pub fn event_kind_index(name: &str) -> usize {
    crate::variant_index(EVENT_KIND_VARIANTS, name)
}

pub fn mouse_action_index(name: &str) -> usize {
    crate::variant_index(MOUSE_ACTION_VARIANTS, name)
}

pub fn mouse_button_index(name: &str) -> usize {
    crate::variant_index(MOUSE_BUTTON_VARIANTS, name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleEvent {
    pub kind: usize,
    pub key: ConsoleKeyEvent,
    pub mouse_action: usize,
    pub mouse_button: usize,
    pub mouse_x: i64,
    pub mouse_y: i64,
    pub width: i64,
    pub height: i64,
    pub text: String,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl ConsoleEvent {
    pub fn key(key: ConsoleKeyEvent) -> Self {
        Self {
            kind: event_kind_index("Key"),
            shift: key.shift,
            ctrl: key.ctrl,
            alt: key.alt,
            meta: key.meta,
            key,
            mouse_action: mouse_action_index("Unknown"),
            mouse_button: mouse_button_index("None"),
            mouse_x: 0,
            mouse_y: 0,
            width: 0,
            height: 0,
            text: String::new(),
        }
    }

    // Modifier arguments (shift/ctrl/alt/meta) are intentionally flat here; a
    // dedicated `Modifiers` struct would reduce arity but requires a larger refactor.
    #[expect(clippy::too_many_arguments, reason = "8 args represent discrete mouse-event fields; grouping into a Modifiers struct is a future refactor")]
    pub fn mouse(
        action: usize,
        button: usize,
        x: i64,
        y: i64,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    ) -> Self {
        Self {
            kind: event_kind_index("Mouse"),
            key: ConsoleKeyEvent::new(key_kind_index("Unknown"), '\0', false, false, false, false),
            mouse_action: action,
            mouse_button: button,
            mouse_x: x,
            mouse_y: y,
            width: 0,
            height: 0,
            text: String::new(),
            shift,
            ctrl,
            alt,
            meta,
        }
    }

    pub fn resize(width: i64, height: i64) -> Self {
        Self {
            kind: event_kind_index("Resize"),
            key: ConsoleKeyEvent::new(key_kind_index("Unknown"), '\0', false, false, false, false),
            mouse_action: mouse_action_index("Unknown"),
            mouse_button: mouse_button_index("None"),
            mouse_x: 0,
            mouse_y: 0,
            width,
            height,
            text: String::new(),
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    pub fn paste(text: String) -> Self {
        Self {
            kind: event_kind_index("Paste"),
            key: ConsoleKeyEvent::new(key_kind_index("Unknown"), '\0', false, false, false, false),
            mouse_action: mouse_action_index("Unknown"),
            mouse_button: mouse_button_index("None"),
            mouse_x: 0,
            mouse_y: 0,
            width: 0,
            height: 0,
            text,
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    pub fn focus_gained() -> Self {
        Self {
            kind: event_kind_index("FocusGained"),
            key: ConsoleKeyEvent::new(key_kind_index("Unknown"), '\0', false, false, false, false),
            mouse_action: mouse_action_index("Unknown"),
            mouse_button: mouse_button_index("None"),
            mouse_x: 0,
            mouse_y: 0,
            width: 0,
            height: 0,
            text: String::new(),
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    pub fn focus_lost() -> Self {
        Self {
            kind: event_kind_index("FocusLost"),
            key: ConsoleKeyEvent::new(key_kind_index("Unknown"), '\0', false, false, false, false),
            mouse_action: mouse_action_index("Unknown"),
            mouse_button: mouse_button_index("None"),
            mouse_x: 0,
            mouse_y: 0,
            width: 0,
            height: 0,
            text: String::new(),
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }
}
