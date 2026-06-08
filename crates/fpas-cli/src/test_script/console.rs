//! Map scripted console events to `fpas_std::ConsoleEvent`.
//!
//! **Documentation:** [`docs/future/test-framework/scripted-input.md`](../../../docs/future/test-framework/scripted-input.md),
//! [`docs/pascal/std/console.md`](../../../docs/pascal/std/console.md).

use fpas_std::key_event::key_kind_index;
use fpas_std::{
    ConsoleEvent, ConsoleKeyEvent, KEY_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS,
    mouse_action_index, mouse_button_index,
};

use super::parse::ScriptEvent;

/// Builds a console event from a parsed script event.
pub fn console_event_from_script(event: &ScriptEvent) -> Result<ConsoleEvent, String> {
    match event {
        ScriptEvent::ConsoleKey {
            kind,
            ch,
            shift,
            ctrl,
            alt,
            meta,
        } => {
            let kind_index = parse_key_kind(kind)?;
            let ch = ch.unwrap_or('\0');
            Ok(ConsoleEvent::key(ConsoleKeyEvent::new(
                kind_index, ch, *shift, *ctrl, *alt, *meta,
            )))
        }
        ScriptEvent::ConsoleMouse {
            action,
            button,
            x,
            y,
            shift,
            ctrl,
            alt,
            meta,
        } => Ok(ConsoleEvent::mouse(
            parse_mouse_action(action)?,
            parse_mouse_button(button)?,
            *x,
            *y,
            *shift,
            *ctrl,
            *alt,
            *meta,
        )),
        ScriptEvent::ConsoleResize { width, height } => Ok(ConsoleEvent::resize(*width, *height)),
        ScriptEvent::ConsolePaste { text } => Ok(ConsoleEvent::paste(text.clone())),
        ScriptEvent::ConsoleFocusGained => Ok(ConsoleEvent::focus_gained()),
        ScriptEvent::ConsoleFocusLost => Ok(ConsoleEvent::focus_lost()),
        _ => Err("internal error: not a console script event".to_string()),
    }
}

fn parse_key_kind(name: &str) -> Result<usize, String> {
    for variant in KEY_KIND_VARIANTS {
        if variant.eq_ignore_ascii_case(name) {
            return Ok(key_kind_index(variant));
        }
    }
    Err(format!(
        "unknown key kind `{name}`\n  help: use names from KeyKind in docs/pascal/std/console.md (e.g. Escape, Enter, Character)."
    ))
}

fn parse_mouse_action(name: &str) -> Result<usize, String> {
    for variant in MOUSE_ACTION_VARIANTS {
        if variant.eq_ignore_ascii_case(name) {
            return Ok(mouse_action_index(variant));
        }
    }
    Err(format!(
        "unknown mouse action `{name}`\n  help: use Down, Up, Move, or other MouseAction variants from docs/pascal/std/console.md."
    ))
}

fn parse_mouse_button(name: &str) -> Result<usize, String> {
    for variant in MOUSE_BUTTON_VARIANTS {
        if variant.eq_ignore_ascii_case(name) {
            return Ok(mouse_button_index(variant));
        }
    }
    Err(format!(
        "unknown mouse button `{name}`\n  help: use Left, Right, Middle, or None."
    ))
}
