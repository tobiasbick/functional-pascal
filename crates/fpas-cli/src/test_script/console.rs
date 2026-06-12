//! Map scripted console events to `fpas_std::ConsoleEvent`.
//!
//! **Documentation:** [`docs/future/test-framework/scripted-input.md`](../../../docs/future/test-framework/scripted-input.md),
//! [`docs/pascal/std/console.md`](../../../docs/pascal/std/console.md).

use fpas_std::{ConsoleEvent, ConsoleKeyEvent};

use super::input::{parse_key_kind, parse_mouse_action, parse_mouse_button};
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
