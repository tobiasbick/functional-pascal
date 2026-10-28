//! Console key event data passed to the executing program.

pub use fpas_builtins::key_event::{KEY_KIND_VARIANTS, key_kind_index};

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
