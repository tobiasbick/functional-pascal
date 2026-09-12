//! DAP mapping for queued debuggee input, EOF, and cancel.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::{DebugOp, ResponseBody};

pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    match (command, body) {
        (
            "fpas/input",
            ResponseBody::InputQueued {
                bytes,
                session_bytes,
            },
        ) => Some(json!({
            "bytes": bytes,
            "sessionBytes": session_bytes
        })),
        ("fpas/eof", ResponseBody::Eof) => Some(json!({"eof": true})),
        ("fpas/cancelInput", ResponseBody::Cleared) => Some(json!({"cleared": true})),
        (
            "fpas/terminalInput",
            ResponseBody::InputQueued {
                bytes,
                session_bytes,
            },
        ) => Some(json!({"bytes": bytes, "sessionBytes": session_bytes})),
        _ => None,
    }
}

impl DapServer {
    pub(super) fn poll_terminal(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        let mut messages = self.terminal_output_events();
        messages.push(self.success(request_seq, command, json!({})));
        messages
    }

    pub(super) fn push_terminal_input(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match terminal_events(arguments) {
            Ok(events) => {
                self.core_request(request_seq, command, DebugOp::TerminalInput { events })
            }
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }

    pub(super) fn push_debuggee_input(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match args::required_string(arguments, "text") {
            Ok(text) => self.core_request(request_seq, command, DebugOp::IoInput { text }),
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }

    pub(super) fn signal_debuggee_eof(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, DebugOp::IoEof)
    }

    pub(super) fn cancel_debuggee_input(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, DebugOp::IoCancel)
    }
}

fn terminal_events(arguments: &Value) -> Result<Vec<fpas_vm::DebugTerminalEvent>, String> {
    let values = arguments
        .get("events")
        .and_then(Value::as_array)
        .ok_or_else(|| "DAP terminal input requires an `events` array.".to_string())?;
    if values.is_empty() || values.len() > 256 {
        return Err("DAP terminal input requires between 1 and 256 events.".to_string());
    }
    values.iter().map(terminal_event).collect()
}

fn terminal_event(value: &Value) -> Result<fpas_vm::DebugTerminalEvent, String> {
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| "Each terminal event requires a string `kind`.".to_string())?;
    match kind {
        "key" => Ok(fpas_vm::DebugTerminalEvent::Key(
            fpas_vm::DebugTerminalKeyEvent {
                kind: terminal_key(value)?,
                shift: flag(value, "shift"),
                ctrl: flag(value, "ctrl"),
                alt: flag(value, "alt"),
                meta: flag(value, "meta"),
            },
        )),
        "mouse" => Ok(fpas_vm::DebugTerminalEvent::Mouse {
            action: mouse_action(required_string(value, "action")?)?,
            button: mouse_button(required_string(value, "button")?)?,
            x: positive_u16(value, "x")?,
            y: positive_u16(value, "y")?,
            shift: flag(value, "shift"),
            ctrl: flag(value, "ctrl"),
            alt: flag(value, "alt"),
            meta: flag(value, "meta"),
        }),
        "resize" => Ok(fpas_vm::DebugTerminalEvent::Resize {
            width: positive_u16(value, "width")?,
            height: positive_u16(value, "height")?,
        }),
        "paste" => Ok(fpas_vm::DebugTerminalEvent::Paste(
            required_string(value, "text")?.to_string(),
        )),
        "focusGained" => Ok(fpas_vm::DebugTerminalEvent::FocusGained),
        "focusLost" => Ok(fpas_vm::DebugTerminalEvent::FocusLost),
        other => Err(format!(
            "Unsupported terminal event kind `{other}`; use key, mouse, resize, paste, focusGained, or focusLost."
        )),
    }
}

fn terminal_key(value: &Value) -> Result<fpas_vm::DebugTerminalKeyKind, String> {
    let key = required_string(value, "key")?;
    let kind = match key {
        "Escape" => fpas_vm::DebugTerminalKeyKind::Escape,
        "Tab" => fpas_vm::DebugTerminalKeyKind::Tab,
        "Enter" => fpas_vm::DebugTerminalKeyKind::Enter,
        "Backspace" => fpas_vm::DebugTerminalKeyKind::Backspace,
        "Space" => fpas_vm::DebugTerminalKeyKind::Space,
        "Up" => fpas_vm::DebugTerminalKeyKind::Up,
        "Down" => fpas_vm::DebugTerminalKeyKind::Down,
        "Left" => fpas_vm::DebugTerminalKeyKind::Left,
        "Right" => fpas_vm::DebugTerminalKeyKind::Right,
        "Home" => fpas_vm::DebugTerminalKeyKind::Home,
        "End" => fpas_vm::DebugTerminalKeyKind::End,
        "PageUp" => fpas_vm::DebugTerminalKeyKind::PageUp,
        "PageDown" => fpas_vm::DebugTerminalKeyKind::PageDown,
        "Insert" => fpas_vm::DebugTerminalKeyKind::Insert,
        "Delete" => fpas_vm::DebugTerminalKeyKind::Delete,
        key if key.starts_with('F') => {
            let number = key[1..]
                .parse::<u8>()
                .ok()
                .filter(|number| (1..=12).contains(number))
                .ok_or_else(|| format!("Unsupported terminal key `{key}`."))?;
            fpas_vm::DebugTerminalKeyKind::Function(number)
        }
        "Character" => {
            let text = required_string(value, "text")?;
            let mut characters = text.chars();
            let character = characters
                .next()
                .filter(|_| characters.next().is_none())
                .ok_or_else(|| {
                    "Terminal Character events require one Unicode scalar in `text`.".to_string()
                })?;
            fpas_vm::DebugTerminalKeyKind::Character(character)
        }
        other => return Err(format!("Unsupported terminal key `{other}`.")),
    };
    Ok(kind)
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Terminal event requires a string `{field}`."))
}

fn positive_u16(value: &Value, field: &str) -> Result<u16, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|number| u16::try_from(number).ok())
        .filter(|number| *number > 0)
        .ok_or_else(|| format!("Terminal event requires `{field}` in 1..=65535."))
}

fn flag(value: &Value, field: &str) -> bool {
    value.get(field).and_then(Value::as_bool).unwrap_or(false)
}

fn mouse_action(value: &str) -> Result<fpas_vm::DebugTerminalMouseAction, String> {
    match value {
        "Down" => Ok(fpas_vm::DebugTerminalMouseAction::Down),
        "Up" => Ok(fpas_vm::DebugTerminalMouseAction::Up),
        "Drag" => Ok(fpas_vm::DebugTerminalMouseAction::Drag),
        "Move" => Ok(fpas_vm::DebugTerminalMouseAction::Move),
        "ScrollDown" => Ok(fpas_vm::DebugTerminalMouseAction::ScrollDown),
        "ScrollUp" => Ok(fpas_vm::DebugTerminalMouseAction::ScrollUp),
        "ScrollLeft" => Ok(fpas_vm::DebugTerminalMouseAction::ScrollLeft),
        "ScrollRight" => Ok(fpas_vm::DebugTerminalMouseAction::ScrollRight),
        other => Err(format!("Unsupported terminal mouse action `{other}`.")),
    }
}

fn mouse_button(value: &str) -> Result<fpas_vm::DebugTerminalMouseButton, String> {
    match value {
        "None" => Ok(fpas_vm::DebugTerminalMouseButton::None),
        "Left" => Ok(fpas_vm::DebugTerminalMouseButton::Left),
        "Right" => Ok(fpas_vm::DebugTerminalMouseButton::Right),
        "Middle" => Ok(fpas_vm::DebugTerminalMouseButton::Middle),
        other => Err(format!("Unsupported terminal mouse button `{other}`.")),
    }
}
