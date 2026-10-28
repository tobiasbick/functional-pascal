//! Standard console event and color discriminants.
//!
//! Spec: `docs/pascal/std/console/README.md` (from repository root).

/// Ordered variants of `Std.Console.ColorKind`.
pub const CONSOLE_COLOR_KIND_VARIANTS: &[&str] = &["Crt", "Ansi256", "Rgb"];

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
