//! Map scripted graph events to `fpas_std::GraphEvent`.
//!
//! **Documentation:** [`docs/future/test-framework/scripted-input.md`](../../../docs/future/test-framework/scripted-input.md),
//! [`docs/pascal/std/graph.md`](../../../docs/pascal/std/graph.md).

use fpas_std::{ConsoleKeyEvent, GraphEvent};

use super::input::{parse_key_kind, parse_mouse_action, parse_mouse_button};
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
