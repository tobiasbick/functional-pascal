//! Map scripted graph events to `fpas_std::GraphEvent`.
//!
//! **Documentation:** [`docs/future/test-framework/scripted-input.md`](../../../docs/future/test-framework/scripted-input.md),
//! [`docs/pascal/std/graph.md`](../../../docs/pascal/std/graph.md).

use fpas_std::key_event::key_kind_index;
use fpas_std::{
    ConsoleKeyEvent, GraphEvent, KEY_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS,
    mouse_action_index, mouse_button_index,
};

use super::parse::ScriptEvent;

/// Builds a graph event from a parsed script event.
pub fn graph_event_from_script(event: &ScriptEvent) -> Result<GraphEvent, String> {
    match event {
        ScriptEvent::GraphKey {
            kind,
            ch,
            shift,
            ctrl,
            alt,
            meta,
        } => {
            let kind_index = parse_key_kind(kind)?;
            let ch = ch.unwrap_or('\0');
            Ok(GraphEvent::Key(ConsoleKeyEvent::new(
                kind_index, ch, *shift, *ctrl, *alt, *meta,
            )))
        }
        ScriptEvent::GraphMouse {
            action,
            button,
            x,
            y,
            shift,
            ctrl,
            alt,
            meta,
        } => Ok(GraphEvent::Mouse {
            action: parse_mouse_action(action)?,
            button: parse_mouse_button(button)?,
            x: *x,
            y: *y,
            shift: *shift,
            ctrl: *ctrl,
            alt: *alt,
            meta: *meta,
        }),
        ScriptEvent::GraphWheel {
            delta_x,
            delta_y,
            shift,
            ctrl,
            alt,
            meta,
        } => Ok(GraphEvent::Wheel {
            delta_x: *delta_x,
            delta_y: *delta_y,
            x: 0,
            y: 0,
            shift: *shift,
            ctrl: *ctrl,
            alt: *alt,
            meta: *meta,
        }),
        _ => Err("internal error: not a graph script event".to_string()),
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
