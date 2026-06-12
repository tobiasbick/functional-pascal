//! Shared script input token parsing for console and graph events.

use fpas_std::key_event::key_kind_index;
use fpas_std::{
    KEY_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS, mouse_action_index,
    mouse_button_index,
};

/// Resolves a `KeyKind` variant name to its runtime index.
pub(super) fn parse_key_kind(name: &str) -> Result<usize, String> {
    for variant in KEY_KIND_VARIANTS {
        if variant.eq_ignore_ascii_case(name) {
            return Ok(key_kind_index(variant));
        }
    }
    Err(format!(
        "unknown key kind `{name}`\n  help: use names from KeyKind in docs/pascal/std/console.md (e.g. Escape, Enter, Character)."
    ))
}

/// Resolves a `MouseAction` variant name to its runtime index.
pub(super) fn parse_mouse_action(name: &str) -> Result<usize, String> {
    for variant in MOUSE_ACTION_VARIANTS {
        if variant.eq_ignore_ascii_case(name) {
            return Ok(mouse_action_index(variant));
        }
    }
    Err(format!(
        "unknown mouse action `{name}`\n  help: use Down, Up, Move, or other MouseAction variants from docs/pascal/std/console.md."
    ))
}

/// Resolves a `MouseButton` variant name to its runtime index.
pub(super) fn parse_mouse_button(name: &str) -> Result<usize, String> {
    for variant in MOUSE_BUTTON_VARIANTS {
        if variant.eq_ignore_ascii_case(name) {
            return Ok(mouse_button_index(variant));
        }
    }
    Err(format!(
        "unknown mouse button `{name}`\n  help: use Left, Right, Middle, or None."
    ))
}
