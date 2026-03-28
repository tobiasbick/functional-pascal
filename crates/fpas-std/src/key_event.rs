//! `Std.Console.KeyKind` discriminant order — must match `fpas-sema` / `fpas-compiler` registration and `KeyInput` mapping.

/// Ordered variant names; runtime discriminant is the index as `integer` (same as legacy enum encoding).
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
    "End",
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
    crate::variant_index(KEY_KIND_VARIANTS, name)
}

/// One console key event (Rust side); VM maps this to `Std.Console.KeyEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleKeyEvent {
    pub kind: usize,
    pub ch: char,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl ConsoleKeyEvent {
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
