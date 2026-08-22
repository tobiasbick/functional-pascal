//! `Std.Console.KeyKind` discriminant order — must match `fpas-sema` / `fpas-compiler` registration and `KeyInput` mapping.

/// Ordered variant names; the runtime discriminant is the index as `integer`.
pub const KEY_KIND_VARIANTS: &[&str] = &[
    "Unknown",
    "Escape",
    "Tab",
    "Enter",
    "Backspace",
    "Space",
    "Up",
    "Down",
    "Left",
    "Right",
    "Home",
    "EndKey",
    "PageUp",
    "PageDown",
    "Insert",
    "Delete",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "Character",
];

/// Discriminant index for a variant name (`Unknown` if missing).
pub fn key_kind_index(name: &str) -> usize {
    crate::variant_index(KEY_KIND_VARIANTS, name).unwrap_or(0)
}

/// One console key event (Rust side); VM maps this to `Std.Console.KeyEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleKeyEvent {
    /// Index into [`KEY_KIND_VARIANTS`].
    pub kind: usize,
    /// Character payload for `Character` key events.
    pub ch: char,
    /// Whether the Shift modifier was active.
    pub shift: bool,
    /// Whether the Control modifier was active.
    pub ctrl: bool,
    /// Whether the Alt modifier was active.
    pub alt: bool,
    /// Whether the platform Meta modifier was active.
    pub meta: bool,
}

impl ConsoleKeyEvent {
    /// Creates a key event from its discriminant, character, and modifiers.
    pub fn new(kind: usize, ch: char, shift: bool, ctrl: bool, alt: bool, meta: bool) -> Self {
        Self {
            kind,
            ch,
            shift,
            ctrl,
            alt,
            meta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_kind_index_known_variants() {
        assert_eq!(key_kind_index("Unknown"), 0);
        assert_eq!(key_kind_index("Space"), 5);
        assert_eq!(key_kind_index("EndKey"), 11);
        assert_eq!(key_kind_index("F1"), 16);
        let last = KEY_KIND_VARIANTS.len() - 1;
        assert_eq!(key_kind_index("Character"), last);
    }

    #[test]
    fn key_kind_index_unknown_name_is_unknown_discriminant() {
        assert_eq!(key_kind_index("NotAVariant"), 0);
        assert_eq!(key_kind_index(""), 0);
    }
}
