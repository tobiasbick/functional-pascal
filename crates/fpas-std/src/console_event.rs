//! `Std.Console` event model for later TUI-style input handling.
//!
//! Spec: `docs/pascal/std/console/README.md` (from repository root).

use crate::key_event::{ConsoleKeyEvent, key_kind_index};

/// Ordered names whose indices encode `Std.Console.EventKind` values.
pub const EVENT_KIND_VARIANTS: &[&str] = &[
    "Key",
    "Mouse",
    "Resize",
    "Paste",
    "FocusGained",
    "FocusLost",
];

/// Ordered names whose indices encode `Std.Console.MouseAction` values.
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

/// Ordered names whose indices encode `Std.Console.MouseButton` values.
pub const MOUSE_BUTTON_VARIANTS: &[&str] = &["None", "Left", "Right", "Middle"];

/// Returns the event-kind discriminant for `name`.
pub fn event_kind_index(name: &str) -> usize {
    // EventKind has no Unknown variant; fall back to FocusLost (last) rather than Key (first).
    crate::variant_index(EVENT_KIND_VARIANTS, name).unwrap_or(EVENT_KIND_VARIANTS.len() - 1)
}

/// Returns the mouse-action discriminant for `name`, or `Unknown`.
pub fn mouse_action_index(name: &str) -> usize {
    crate::variant_index(MOUSE_ACTION_VARIANTS, name).unwrap_or(0)
}

/// Returns the mouse-button discriminant for `name`, or `None`.
pub fn mouse_button_index(name: &str) -> usize {
    crate::variant_index(MOUSE_BUTTON_VARIANTS, name).unwrap_or(0)
}

/// Unified key, mouse, resize, paste, or focus event exposed by `Std.Console`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleEvent {
    /// Index into [`EVENT_KIND_VARIANTS`].
    pub kind: usize,
    /// Key data when [`Self::kind`] denotes a key event.
    pub key: ConsoleKeyEvent,
    /// Index into [`MOUSE_ACTION_VARIANTS`] for mouse events.
    pub mouse_action: usize,
    /// Index into [`MOUSE_BUTTON_VARIANTS`] for mouse events.
    pub mouse_button: usize,
    /// One-based mouse column for mouse events.
    pub mouse_x: i64,
    /// One-based mouse row for mouse events.
    pub mouse_y: i64,
    /// Terminal width for resize events.
    pub width: i64,
    /// Terminal height for resize events.
    pub height: i64,
    /// Pasted text for paste events.
    pub text: String,
    /// Whether the Shift modifier was active.
    pub shift: bool,
    /// Whether the Control modifier was active.
    pub ctrl: bool,
    /// Whether the Alt modifier was active.
    pub alt: bool,
    /// Whether the platform Meta modifier was active.
    pub meta: bool,
}

impl ConsoleEvent {
    /// Creates a unified key event.
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
    #[expect(
        clippy::too_many_arguments,
        reason = "8 args represent discrete mouse-event fields; grouping into a Modifiers struct is a future refactor"
    )]
    /// Creates a unified mouse event.
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

    /// Creates a terminal-resize event.
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

    /// Creates a bracketed-paste event.
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

    /// Creates a terminal-focus-gained event.
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

    /// Creates a terminal-focus-lost event.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_kind_index_known_variants() {
        assert_eq!(event_kind_index("Key"), 0);
        assert_eq!(event_kind_index("FocusLost"), EVENT_KIND_VARIANTS.len() - 1);
    }

    #[test]
    fn event_kind_index_unknown_name_is_focus_lost_not_key() {
        assert_eq!(
            event_kind_index("NotAVariant"),
            EVENT_KIND_VARIANTS.len() - 1
        );
        assert_ne!(event_kind_index("NotAVariant"), event_kind_index("Key"));
    }

    #[test]
    fn mouse_action_unknown_name_is_unknown_discriminant() {
        assert_eq!(mouse_action_index("NotAVariant"), 0);
    }
}
